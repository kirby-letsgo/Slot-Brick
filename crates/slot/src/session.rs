use std::path::PathBuf;
use std::time::Duration;

use slot_input::{Action, Gestures, Millis, RawEvent};
use slot_retro::Rumble;
use slot_store::Platform;
use slot_ui::FfState;

pub const RUMBLE_HOLD_MS: Millis = 100;

use crate::app::{App, Phase};
use crate::audio::{AudioSink, Ring, Sfx};
use crate::core::open_core;
use crate::emu::{CoreState, EmuHandle, Speed};
use crate::frames::FrameRef;
use crate::input::Pad;
use crate::persist;
use crate::prefetch::Prefetch;

enum NativeLaunch { Pico8(crate::pico8::Launch), Port(crate::ports::Launch), Dreamcast(crate::dreamcast::Launch) }

pub struct Session {
    root: PathBuf,
    app: App,
    emu: Option<EmuHandle>,
    native: Option<NativeLaunch>,
    sink: Box<dyn AudioSink>,
    gestures: Gestures,
    pad: Pad,
    rewinding: bool,
    fast: bool,
    motor: u16,
    pulse: Option<(Millis, u16)>,
    reloading: bool,
    driven: bool,
    prefetch: Prefetch,
}

impl Session {
    pub fn boot(root: PathBuf) -> Self {
        let sink = crate::audio::take_opened();
        Session {
            app: App::take_booted(&root),
            root,
            emu: None,
            native: None,
            sink,
            gestures: if cfg!(feature = "brick") { Gestures::brick() } else { Gestures::new() },
            prefetch: Prefetch::default(),
            pad: Pad::default(),
            rewinding: false,
            fast: false,
            motor: 0,
            pulse: None,
            reloading: false,
            driven: false,
        }
    }

    pub fn native_ready(&self) -> bool {
        self.native.is_some() && matches!(self.app.phase(), Phase::Playing { .. })
    }

    pub fn queue_native(&mut self) -> Result<(), String> {
        let launch = self.native.take().ok_or("No native game to launch")?;
        let stale = match &launch {
            NativeLaunch::Pico8(_) => ["Config/port-launch", crate::dreamcast::REQUEST],
            NativeLaunch::Port(_) => ["Config/pico8-launch", crate::dreamcast::REQUEST],
            NativeLaunch::Dreamcast(_) => ["Config/pico8-launch", "Config/port-launch"],
        };
        for file in stale {
            match std::fs::remove_file(self.root.join(file)) {
                Ok(()) => {}, Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
                Err(e) => return Err(e.to_string()),
            }
        }
        match launch {
            NativeLaunch::Pico8(l) => crate::pico8::queue(&self.root, &l),
            NativeLaunch::Port(l) => crate::ports::queue(&self.root, &l),
            NativeLaunch::Dreamcast(l) => crate::dreamcast::queue(&self.root, &l),
        }
    }

    pub fn play_sfx(&mut self, sfx: Sfx) {
        let ring = self.sink.ring();
        let rate = ring.sample_rate();
        if rate == 0 {
            return;
        }
        let mut samples = sfx.render(rate);
        crate::audio::volume::apply(&mut samples, self.app.output_volume());
        ring.mix(&samples);
    }

    pub fn audio_queued(&self) -> usize {
        self.sink.ring().queued_frames()
    }

    pub fn audio_ring(&self) -> std::sync::Arc<Ring> {
        self.sink.ring()
    }

    pub fn rumble(&mut self, strength: u16) {
        if strength == self.motor {
            return;
        }
        self.motor = strength;
        self.app.set_rumble(strength);
    }

    pub fn core_rumble(&self) -> Option<&Rumble> {
        self.emu.as_ref().map(EmuHandle::rumble)
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn emu(&self) -> Option<&EmuHandle> {
        self.emu.as_ref()
    }

    pub fn set_driven(&mut self, driven: bool) {
        self.driven = driven;
        if let Some(emu) = &self.emu {
            emu.set_driven(driven);
        }
    }

    pub fn step_emulator(&self, present: Duration, timeout: Duration) -> bool {
        self.emu.as_ref().is_some_and(|emu| {
            emu.tick(present);
            emu.wait_frame(timeout)
        })
    }

    pub fn frame(&self) -> Option<FrameRef> {
        self.emu.as_ref().and_then(|e| e.latest_frame())
    }

    pub fn core_settling(&self) -> bool {
        self.emu
            .as_ref()
            .is_some_and(|e| e.state() == CoreState::Loading)
    }

    pub fn has_core(&self) -> bool {
        self.emu.is_some()
    }

    pub fn frame_ready(&self) -> bool {
        self.emu.as_ref().is_some_and(EmuHandle::frame_ready)
    }

    pub fn frames_published(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::published_count)
    }

    /// Executed core frames (also those omitted by fast-forward frame skipping).
    pub fn emulated_frames(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::executed_count)
    }

    pub fn observed_speed(&self) -> Option<Speed> {
        self.emu.as_ref().map(EmuHandle::observed_speed)
    }

    pub fn frames_taken(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::frames_taken)
    }

    pub fn game_visible(&self) -> bool {
        self.app.game_visible()
    }

    pub fn feed(&mut self, events: impl IntoIterator<Item = RawEvent>, now: Millis) {
        self.gestures.set_triggers_to_game(
            self.has_core() && matches!(self.app.platform(), Platform::Ps1 | Platform::N64 | Platform::Pce | Platform::PceCd | Platform::Ws),
        );
        let mut actions = Vec::new();
        for ev in events {
            actions.extend(self.gestures.feed(ev, now));
        }
        actions.extend(self.gestures.tick(now));
        for action in actions {
            self.act(action);
        }
        self.sync_pad();
    }

    #[cfg(feature = "brick")]
    pub fn reset_sleep_input(&mut self) {
        self.gestures = Gestures::brick();
        self.pad.clear();
        self.rewinding = false;
        self.fast = false;
        self.sync_pad();
    }

    /// A PlayStation game takes a Brick Pro's stick as its DualShock's left stick.
    pub fn stick_to_game(&self) -> bool {
        self.has_core()
            && matches!(self.app.platform(), Platform::Ps1 | Platform::N64)
            && self.app.game_visible()
    }

    pub fn set_stick(&mut self, at: Option<(i16, i16)>) {
        if let (Some(emu), true) = (&self.emu, self.stick_to_game()) {
            emu.set_stick(at.unwrap_or_default());
        }
    }

    fn sync_pad(&mut self) {
        for btn in self.app.taken_buttons() {
            self.pad.apply(Action::GbaUp(*btn));
        }
        if let Some(emu) = &self.emu {
            emu.set_input(self.pad.mask());
        }
    }

    fn bridge_link(&mut self, f: impl FnOnce(&mut App)) {
        let had_link = self.app.link_active();
        f(&mut self.app);
        if had_link && !self.app.link_active() {
            if let Some(emu) = &self.emu {
                emu.end_link();
            }
        }
    }

    fn act(&mut self, action: Action) {
        // Keep the existing Select+B shortcut as an alias on Brick.
        #[cfg(feature = "brick")]
        let action = if action == Action::ShaderPrev { Action::LedToggle } else { action };
        if trace() {
            eprintln!("slot: {action:?} in {:?}", self.app.phase());
        }
        match action {
            Action::RewindStart => self.rewinding = true,
            Action::RewindStop => self.rewinding = false,
            Action::FfStart => self.fast = true,
            Action::FfStop => self.fast = false,
            _ => {}
        }
        let menu = self.overlaid();
        self.bridge_link(|app| app.apply(action));
        if matches!(
            action,
            Action::VolumeUp | Action::VolumeDown | Action::MuteToggle
        ) {
            if let Some(emu) = &self.emu {
                emu.set_volume(self.app.output_volume());
            }
        }
        if menu || self.overlaid() {
            self.pad.clear();
        } else if self.app.takes_from_the_game(action) {
            if let Action::GbaDown(btn) | Action::GbaUp(btn) = action {
                self.pad.apply(Action::GbaUp(btn));
            }
        } else {
            self.pad.apply(action);
        }
        self.sync_rumble();
    }

    pub fn update(&mut self, dt: f32) {
        self.bridge_link(|app| app.update(dt));
        if let Some(emu) = &self.emu {
            emu.set_volume(self.app.output_volume());
            crate::badge::prefetch(&self.root, emu.take_badge_urls());
            for notice in emu.take_achievement_notices() {
                let badge = match &notice {
                    slot_cheevos::Notice::Unlocked { badge, .. }
                    | slot_cheevos::Notice::GameCompleted { badge, .. }
                    | slot_cheevos::Notice::GameLoaded(Ok((_, badge))) => Some(badge.clone()),
                    _ => None,
                };
                let badge = badge.and_then(|url| {
                    crate::badge::prefetch(&self.root, vec![url.clone()]);
                    crate::badge::path(&self.root, &url)
                });
                self.app.achievement_notice(notice, badge);
            }
        }
        if let Some((client_id, transport)) = self.app.take_link_transport() {
            match &self.emu {
                Some(emu) => match self.app.link_player() {
                    Some(player) => emu.begin_cable(player, transport),
                    None => emu.begin_link(client_id, transport),
                },
                None => eprintln!("slot: link: a transport arrived with no core to run it"),
            }
        }
        if let Some(p) = self.app.take_gb_palette() {
            if let Some(emu) = &self.emu {
                emu.set_option("mgba_gb_colors", p.core_name());
            }
        }
        if let Some(on) = self.app.take_colour_correction() {
            if let Some((key, value)) = crate::core::colour_option(self.app.core(), on) {
                if let Some(emu) = &self.emu {
                    emu.set_option(key, value);
                }
            }
        }
        if let Some(on) = self.app.take_ps1_dithering() {
            if let (Some(emu), Platform::Ps1) = (&self.emu, self.app.platform()) {
                emu.set_option("pcsx_rearmed_dithering", crate::core::dithering(on));
            }
        }
        if let Some((stem, serial)) = self.app.take_link_reload() {
            self.reload_for_link(&stem, serial);
        }
        if self.app.link_active() {
            if self.emu.as_ref().is_some_and(EmuHandle::bios_mismatch) {
                self.bridge_link(|app| app.bios_mismatch());
            } else if self.emu.as_ref().is_some_and(EmuHandle::peer_ended) {
                self.bridge_link(|app| app.peer_ended());
            } else if self.emu.as_ref().is_some_and(EmuHandle::link_lost) {
                self.app.peer_lost();
            }
        }
        if let Some(sfx) = self.app.take_sfx() {
            self.play_sfx(sfx);
        }
        self.sync_core();
        self.sync_reload();
        if !self.reloading {
            if let Some((stem, serial)) = self.app.take_solo_reload() {
                eprintln!("slot: link: restoring single-player mode");
                self.reload_for_link(&stem, serial);
            }
        }
        if !self.has_core() {
            for action in self.gestures.drop_ff_latch() {
                self.act(action);
            }
        }
        self.app
            .set_game_ready(self.emu.as_ref().is_some_and(EmuHandle::has_published));
        self.sync_speed();
        self.sync_rewind_hud();
        self.sync_ff_hud();
        self.sync_rumble();
        self.sync_pad();
        // 1.6.0: resting on a cart reads its ROM ahead, so inserting it does not wait on the card.
        let now = self.app.now();
        self.prefetch.browse(self.app.browsed_rom(), now);
    }

    pub fn prefetching(&self) -> Option<&std::path::Path> {
        self.prefetch.reading()
    }

    fn sync_rumble(&mut self) {
        let asked = match &self.emu {
            Some(emu) if self.playing() && self.app.rumble_enabled() => emu.rumble().strength(),
            _ => {
                self.pulse = None;
                return self.rumble(0);
            }
        };
        let now = self.app.now();
        if asked > 0 {
            self.pulse = Some((now, asked));
        }
        let want = match self.pulse {
            Some((at, strength)) if now.saturating_sub(at) < RUMBLE_HOLD_MS => strength,
            _ => 0,
        };
        self.rumble(want);
    }

    fn sync_ff_hud(&mut self) {
        let ff = match (self.actually_fast_forwarding(), self.gestures.ff_latched()) {
            (false, _) => FfState::Off,
            (true, false) => FfState::Held,
            (true, true) => FfState::Latched,
        };
        self.app.set_ff(ff);
    }

    fn sync_rewind_hud(&mut self) {
        let fill = self
            .actually_rewinding()
            .then(|| self.emu.as_ref().map(EmuHandle::rewind_fill))
            .flatten();
        match fill {
            Some(fill) => self.app.show_rewind(fill),
            None => self.app.hide_rewind(),
        }
    }

    pub(crate) fn actually_rewinding(&self) -> bool {
        self.rewinding && self.playing() && self.app.may_rewind()
    }

    pub(crate) fn actually_fast_forwarding(&self) -> bool {
        self.fast && self.playing() && self.app.may_fast_forward()
    }

    fn inserting(&self) -> bool {
        matches!(self.app.phase(), Phase::Inserting { .. })
    }

    fn showing_polaroids(&self) -> bool {
        matches!(self.app.phase(), Phase::Polaroids { .. })
    }

    fn overlaid(&self) -> bool {
        self.showing_polaroids() || self.held()
    }

    fn playing(&self) -> bool {
        matches!(self.app.phase(), Phase::Playing { .. }) && !self.held()
    }

    fn held(&self) -> bool {
        self.app.power_menu().is_some()
            || self.app.pause_menu().is_some()
            || self.app.game_menu_open()
            || self.app.shutting_down()
    }

    fn dozing(&self) -> bool {
        matches!(self.app.phase(), Phase::Doze { .. })
    }

    fn ejecting(&self) -> bool {
        matches!(self.app.phase(), Phase::Ejecting { .. })
    }

    fn sync_speed(&self) {
        if let Some(emu) = &self.emu {
            emu.set_fast_steps(u32::from(self.app.ff_speed()));
            emu.set_ff_sound(self.app.ff_sound());
            emu.set_speed(
                if self.inserting()
                    || self.ejecting()
                    || self.showing_polaroids()
                    || self.dozing()
                    || (self.held() && !self.app.link_active())
                {
                    Speed::Paused
                } else if self.actually_fast_forwarding() {
                    Speed::Fast
                } else {
                    Speed::Normal
                },
            );
            emu.set_rewinding(self.actually_rewinding());
            emu.set_turbo(self.app.turbo());
            emu.set_rewind_recording(self.app.records_rewind());
        }
    }

    fn no_core() -> bool {
        std::env::var_os("SLOT_NO_CORE").is_some_and(|v| v != "0")
    }

    fn sync_core(&mut self) {
        if Self::no_core() {
            return;
        }
        let stem = match self.app.phase() {
            Phase::Shelf => {
                self.emu = None;
                self.native = None;
                return;
            }
            Phase::Inserting { cart, .. } => cart.clone(),
            _ => return,
        };
        if self
            .app
            .seated_cart()
            .is_some_and(|c| matches!(c.platform, Platform::Pico8 | Platform::Ports | Platform::Dc))
        {
            if self.native.is_none() {
                let cart = self.app.seated_cart().unwrap().clone();
                let prepared = if cart.platform == Platform::Ports {
                    crate::ports::prepare(&self.root, &cart).map(NativeLaunch::Port)
                } else if cart.platform == Platform::Dc {
                    crate::dreamcast::prepare(&self.root, &cart).map(NativeLaunch::Dreamcast)
                } else { crate::pico8::prepare(&self.root, &cart).map(NativeLaunch::Pico8) };
                match prepared {
                    Ok(launch) => {
                        self.app.set_platform(cart.platform);
                        self.app.set_core(slot_store::Core::default_for(cart.platform));
                        self.native = Some(launch);
                        self.app.on_core_ready();
                    }
                    Err(message) => {
                        self.app.announce(match cart.platform {
                            Platform::Ports => "Port setup",
                            Platform::Dc => "Dreamcast setup",
                            _ => "PICO-8 setup",
                        }, &message);
                        self.app.on_core_failed();
                    }
                }
            }
            return;
        }
        if self.emu.is_none() {
            let (_, serial) = self.app.link_mode(&stem);
            self.spawn_core(&stem, serial);
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => {
                if let (true, Some(p), Some(emu)) =
                    (self.app.palette_live(), self.app.gb_palette(), &self.emu)
                {
                    emu.set_option("mgba_gb_colors", p.core_name());
                }
                self.app.on_core_ready()
            }
            Some(CoreState::Failed) | None => {
                let failed_neogeo = self.emu.is_some() && self.app.platform() == Platform::NeoGeo;
                let failed_cd = self.emu.is_some() && self.app.platform() == Platform::SegaCd;
                let failed_pcecd = self.emu.is_some() && self.app.platform() == Platform::PceCd;
                self.emu = None;
                self.app.on_core_failed();
                if failed_neogeo { self.app.announce("Neo Geo could not start", "Check the FBNeo ROM set and neogeo.zip BIOS. Keep original ZIP names. Details are in the log."); }
                if failed_pcecd {
                    self.app.announce("PC Engine CD could not start", "Check the disc files and Slot/BIOS/syscard3.pce.");
                }
                if failed_cd {
                    self.app.announce("Sega CD could not start", "Check the disc files and its region's BIOS in Slot/BIOS.");
                }
            }
        }
    }

    fn spawn_core(&mut self, stem: &str, serial: &'static str) {
        let Some((rom, platform)) = self
            .app
            .seated_cart()
            .filter(|c| c.stem == stem)
            .map(|c| (c.rom.clone(), c.platform))
        else {
            return;
        };
        if platform == Platform::Lynx {
            if let Err(message) = crate::root::check_lynx_bios(&self.root) {
                self.app.on_core_failed();
                self.app.announce("Atari Lynx setup", message);
                return;
            }
        }
        if platform == Platform::NeoGeo && !self.root.join("BIOS/neogeo.zip").is_file()
            && !rom.parent().is_some_and(|p| p.join("neogeo.zip").is_file()) {
            self.app.on_core_failed();
            self.app.announce("Neo Geo setup", "Put the FBNeo neogeo.zip BIOS in Slot/BIOS, then try again.");
            return;
        }
        if platform == Platform::SegaCd {
            if let Err(message) = crate::sega_cd::check_bios(&self.root) {
                self.app.on_core_failed();
                self.app.announce("Sega CD setup", message);
                return;
            }
        }
        if platform == Platform::PceCd {
            if let Err(message) = crate::pce::check_bios(&self.root) {
                self.app.on_core_failed();
                self.app.announce("PC Engine CD setup", message);
                return;
            }
        }
        let core = slot_store::core_for_platform(&self.root, stem, platform);
        self.app.set_core(core);
        self.app.set_platform(platform);
        self.app
            .set_video_mode(crate::video_mode::video_mode_for_platform(
                &self.root, stem, platform,
            ));
        self.app.set_link_loaded(serial);
        let resume = (!self.app.starting_clean())
            .then(|| persist::read_resume(&self.root, platform, core, stem))
            .flatten();
        let player = self.app.link_player();
        let palette = crate::core::palette_for(self.app.gb_palette(), platform, &rom, player);
        self.app.set_palette_live(palette.is_some());
        let mut opened = if platform == Platform::Gg {
            crate::core::open_game_gear(&self.root)
        } else if matches!(platform, Platform::Pce | Platform::PceCd) {
            crate::core::open_pce(&self.root, platform)
        } else if platform == Platform::SegaCd {
            crate::core::open_sega_cd(&self.root)
        } else if platform == Platform::Sms {
            crate::core::open_master_system(&self.root)
        } else {
            open_core(
                &self.root,
                core,
                serial,
                self.app.colour_correction(),
                palette,
                player,
            )
        };
        if matches!(platform, Platform::Ws | Platform::Lynx) && !opened.named {
            self.app.on_core_failed();
            self.app.announce("Emulator setup", "Restore the emulator from the latest Slot card package. HOME > System > System Setup shows the missing file.");
            return;
        }
        if let Some(disc) = self.app.chosen_disc().filter(|_| platform.is_disc()) {
            opened.core.want_disc(disc);
        }
        self.app.set_named_core(opened.named);
        let sav = persist::read_sav(&self.root, platform, stem);
        if let Some(sav) = &sav {
            if let Err(e) = persist::backup_sav(&self.root, platform, stem, sav) {
                eprintln!("slot: save backup: {e}");
            }
        }
        let ring = self.sink.ring();
        let unpack_into = self.root.join(slot_store::archive::UNPACKED_DIR);
        let linked = player.filter(|_| platform == Platform::Gba);
        let emu = if platform == Platform::NeoGeo {
            EmuHandle::spawn(opened.core, rom, ring, sav, resume)
        } else {
            EmuHandle::spawn_game(opened.core, rom, unpack_into, ring, sav, resume, linked)
        };
        emu.set_volume(self.app.output_volume());
        if platform == Platform::Ps1 {
            emu.set_option("pcsx_rearmed_dithering", crate::core::dithering(self.app.ps1_dithering()));
        }
        // The SNES, Mega Drive and PlayStation pads have X and Y of their own: no turbo on them.
        emu.set_native_xy(matches!(
            platform,
            Platform::Snes | Platform::Md | Platform::S32x | Platform::SegaCd | Platform::Pce | Platform::PceCd | Platform::NeoGeo | Platform::Ps1 | Platform::N64 | Platform::Ws
        ));
        emu.set_driven(self.driven);
        // Logged in to RetroAchievements: this game earns achievements (softcore).
        if let Some(account) = crate::ra::load(&self.root) {
            let console = match platform {
                Platform::Gba => slot_cheevos::Console::Gba,
                Platform::Gb => slot_cheevos::Console::Gb,
                Platform::Gbc => slot_cheevos::Console::Gbc,
                Platform::Nes => slot_cheevos::Console::Nes,
                Platform::Snes => slot_cheevos::Console::Snes,
                Platform::Md => slot_cheevos::Console::MegaDrive,
                Platform::S32x => slot_cheevos::Console::Sega32X,
                Platform::SegaCd => slot_cheevos::Console::SegaCd,
                Platform::Pce => slot_cheevos::Console::PcEngine,
                Platform::PceCd => slot_cheevos::Console::PcEngineCd,
                Platform::NeoGeo => slot_cheevos::Console::Arcade,
                Platform::Gg => slot_cheevos::Console::GameGear,
                Platform::Sms => slot_cheevos::Console::MasterSystem,
                Platform::N64 => slot_cheevos::Console::N64,
                Platform::Ngp => slot_cheevos::Console::NeoGeoPocket,
                Platform::Ws => slot_cheevos::Console::WonderSwan,
                Platform::Lynx => slot_cheevos::Console::Lynx,
                Platform::Pico8 | Platform::Ports | Platform::Dc => unreachable!("native runtime has no libretro achievements"),
                Platform::Ps1 => slot_cheevos::Console::Ps1,
            };
            emu.achievements(&account.user, &account.token, console);
        }
        self.app.set_snapshot(Box::new(emu.snapshot()));
        self.emu = Some(emu);
    }

    fn reload_for_link(&mut self, stem: &str, serial: &'static str) {
        eprintln!("slot: link: loading {stem} again with gpsp_serial={serial}");
        if self.emu.is_some() {
            self.app.flush_resume();
        }
        self.emu = None;
        self.spawn_core(stem, serial);
        self.reloading = true;
    }

    fn sync_reload(&mut self) {
        if !self.reloading {
            return;
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => {
                self.reloading = false;
                self.app.link_reload_done();
            }
            Some(CoreState::Failed) | None => {
                self.reloading = false;
                self.emu = None;
                self.app.link_reload_failed();
                if let Some((stem, serial)) = self.app.take_link_reload() {
                    self.reload_for_link(&stem, serial);
                }
            }
        }
    }
}

pub(crate) fn trace() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SLOT_TRACE").is_some())
}
