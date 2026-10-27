//! Brick Wi-Fi settings. Uses the firmware supplicant; no shell interpolation of credentials.
use std::{fs,os::unix::{net::UnixDatagram,fs::PermissionsExt},path::{Path,PathBuf},process::{Command,Stdio},sync::{mpsc,Arc,atomic::{AtomicBool,AtomicU64,Ordering}},thread,time::{Duration,Instant}};
use slot_input::{Action,Btn};
use slot_store::Platform;
use slot_ui::{TexId,UndoFace};
use crate::cart_art::{ArtJob,Fetched,Progress};
use crate::ra;
type Result<T> = std::result::Result<T,String>;
static SERIAL:AtomicU64=AtomicU64::new(0);
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
enum Security {Open,Personal,Unsupported}
#[derive(Clone,Debug)]
struct Network {ssid:Vec<u8>,signal:i32,security:Security,saved:Option<u32>}
fn decode(s:&str)->Vec<u8> {
    let b=s.as_bytes();let mut v=Vec::new();let mut i=0;
    while i<b.len(){
        if b[i]==b'\\' && i+1<b.len(){
            if b[i+1]==b'x' && i+3<b.len(){if let Some(n)=std::str::from_utf8(&b[i+2..i+4]).ok().and_then(|s|u8::from_str_radix(s,16).ok()){v.push(n);i+=4;continue}}
            let c=match b[i+1]{b'\\'=>Some(b'\\'),b'"'=>Some(b'"'),b'n'=>Some(b'\n'),b't'=>Some(b'\t'),b'r'=>Some(b'\r'),_=>None};
            if let Some(c)=c {v.push(c);i+=2;continue}
        }v.push(b[i]);i+=1;
    }v
}
fn name(b:&[u8])->String{String::from_utf8_lossy(b).chars().map(|c|if c.is_control(){' '}else{c}).collect()}
fn hex(b:&[u8])->String{b.iter().map(|b|format!("{b:02x}")).collect()}
fn psk(s:&str)->Result<String>{
    if s.len()==64 && s.bytes().all(|b|b.is_ascii_hexdigit()){return Ok(s.into())}
    if !(8..=63).contains(&s.len()) || !s.bytes().all(|b|(32..=126).contains(&b)){return Err("Use 8-63 characters or 64 hex digits".into())}
    // wpa_config_parse_psk takes the literal bytes between the first and last quote.
    // Backslashes and interior quotes must NOT be C-escaped here.
    Ok(format!("\"{s}\""))
}
fn scan_rows(scan:&str,saved:&str)->Vec<Network>{
    let saved:Vec<_>=saved.lines().skip(1).filter_map(|s|{let v:Vec<_>=s.split('\t').collect();Some((v.first()?.parse::<u32>().ok()?,decode(v.get(1)?)))}).collect();
    let mut out:Vec<Network>=Vec::new();
    for s in scan.lines().skip(1){let v:Vec<_>=s.splitn(5,'\t').collect();if v.len()!=5{continue}let ssid=decode(v[4]);if ssid.is_empty() || ssid.len()>32{continue}
        let security=if v[3].contains("PSK"){Security::Personal}else if ["WPA","WEP","EAP","SAE","OWE","RSN"].iter().any(|s|v[3].contains(s)){Security::Unsupported}else{Security::Open};
        let n=Network{saved:saved.iter().find(|(_,s)|s==&ssid).map(|(id,_)|*id),ssid,security,signal:v[2].parse().unwrap_or(-100)};
        if let Some(old)=out.iter_mut().find(|a|a.ssid==n.ssid && a.security==n.security){if old.signal<n.signal{*old=n}}else{out.push(n)}
    }out.sort_by(|a,b|b.signal.cmp(&a.signal).then(a.ssid.cmp(&b.ssid)));out
}
pub(crate) trait Supplicant {
    fn command(&self,s:&str)->Result<String>;
    fn ok(&self,s:&str)->Result<()>{if self.command(s)?=="OK"{Ok(())}else{Err("Wi-Fi service rejected the request".into())}}
}
pub(crate) struct Control{socket:UnixDatagram,local:PathBuf}
impl Drop for Control{fn drop(&mut self){let _=fs::remove_file(&self.local);}}
impl Control{
    pub(crate) fn at(remote:&Path)->Result<Self>{
        let local=PathBuf::from(format!("/tmp/slot-wifi-{}-{}",std::process::id(),SERIAL.fetch_add(1,Ordering::Relaxed)));
        let socket=UnixDatagram::bind(&local).map_err(|_|"Cannot open Wi-Fi control socket")?;
        let c=Self{socket,local};fs::set_permissions(&c.local,fs::Permissions::from_mode(0o600)).map_err(|_|"Cannot secure Wi-Fi socket")?;
        c.socket.set_read_timeout(Some(Duration::from_secs(2))).map_err(|_|"Cannot set Wi-Fi timeout")?;
        c.socket.set_write_timeout(Some(Duration::from_secs(2))).map_err(|_|"Cannot set Wi-Fi timeout")?;
        c.socket.connect(remote).map_err(|_|"Wi-Fi service unavailable")?;Ok(c)
    }
    fn open()->Result<Self>{SOCKETS.iter().find_map(|s|Self::at(&slot_power::hw(s)).ok()).ok_or("Wi-Fi service unavailable".into())}
}
impl Supplicant for Control{
    fn command(&self,s:&str)->Result<String>{
        self.socket.send(s.as_bytes()).map_err(|_|"Wi-Fi command failed")?;let mut b=[0;32768];let n=self.socket.recv(&mut b).map_err(|_|"Wi-Fi service did not respond")?;
        let s=String::from_utf8_lossy(&b[..n]).trim_end_matches('\n').to_string();
        if s.starts_with("FAIL") || s.starts_with("UNKNOWN"){Err("Wi-Fi service rejected the request".into())}else{Ok(s)}
    }
}
fn field<'a>(s:&'a str,key:&str)->Option<&'a str>{s.lines().find_map(|s|s.strip_prefix(key).and_then(|s|s.strip_prefix('=')))}
fn pause(cancel:&AtomicBool,ms:u64)->Result<()>{for _ in 0..(ms+99)/100{if cancel.load(Ordering::Relaxed){return Err("Cancelled".into())}thread::sleep(Duration::from_millis(100));}Ok(())}
/// Slot's own System/brick_wifi.sh always wins: it writes Logs/wifi.log and tries the
/// firmware's /etc/wifi/wifi_init.sh itself, falling back to its own start when that fails.
/// The firmware helper is only run directly when the card copy is missing.
fn helper(firmware:&Path,root:&Path)->Option<PathBuf>{
    let own=root.join("System/brick_wifi.sh");if own.is_file(){return Some(own)}
    firmware.is_file().then(||firmware.into())
}
fn card_root()->PathBuf{PathBuf::from(std::env::var_os("SLOT_ROOT").unwrap_or_else(||"/mnt/SDCARD/Slot".into()))}
fn service(on:bool,cancel:&AtomicBool)->Result<()>{helper_run(if on{"start"}else{"stop"},cancel)}
/// Control sockets slot can reach a supplicant on, in its search order.
const SOCKETS:[&str;3]=["/etc/wifi/sockets/wlan0","/var/run/wpa_supplicant/wlan0","/run/wpa_supplicant/wlan0"];
/// Wi-Fi is switched on, whether or not it has reached anything. Cheap: it only looks for the
/// supplicant's socket. The shelf asks a few times a second, so it must stay cheap.
pub fn radio_on()->bool{supplicant_up()}
/// A supplicant is running: what the menu reports as Wi-Fi on, known without asking it.
fn supplicant_up()->bool{SOCKETS.iter().any(|s|slot_power::hw(s).exists())}
/// A supplicant that answers: the firmware's socket appears a while before it does.
fn supplicant_answers()->bool{Control::open().and_then(|c|c.command("PING")).is_ok_and(|r|r=="PONG")}
/// One helper at a time: the boot repair and the menu may both ask, and two helpers
/// stopping and starting the same supplicant would undo each other.
static HELPER:std::sync::Mutex<()>=std::sync::Mutex::new(());
/// At startup: Wi-Fi as it was left. The firmware may start a supplicant with the radio still
/// blocked (saved networks are then never joined), or, with slot in place of its menu, start
/// none at all. So wait a little for its own, repair that, or else start Wi-Fi here, as
/// opening the Wi-Fi menu and turning it on would.
/// Never in the way of startup: everything, even reading whether Wi-Fi is wanted, happens on
/// its own thread, and nothing waits for it.
pub fn repair_at_boot(){
    let spawned=thread::Builder::new().name("slot-wifi-boot".into()).spawn(||{
        if !wanted(&card_root()){eprintln!("wifi: boot: Wi-Fi was turned off; leaving it off");return}
        let quiet=AtomicBool::new(false);
        boot_repair(supplicant_answers,online,|up|helper_run(if up{"ensure"}else{"start"},&quiet),|ms|thread::sleep(Duration::from_millis(ms)));
    });
    if let Err(e)=spawned{eprintln!("wifi: boot repair: {e}");}
}
/// Where the Wi-Fi menu's on/off is kept, so startup brings Wi-Fi back as it was.
const CHOICE_FILE:&str="Config/wifi.txt";
/// Wi-Fi was left on: as last turned on or off in the menu, else whenever a network is saved.
fn wanted(root:&Path)->bool{
    match fs::read_to_string(root.join(CHOICE_FILE)).map(|s|s.trim().to_string()).as_deref(){
        Ok("off")=>false,
        Ok("on")=>true,
        _=>[slot_power::hw("/etc/wifi/wpa_supplicant.conf"),root.join("Config/wpa_supplicant.conf")].iter()
            .any(|p|fs::read_to_string(p).is_ok_and(|s|s.lines().any(|l|l.trim_start().starts_with("network="))))
    }
}
fn remember(on:bool){
    let path=card_root().join(CHOICE_FILE);
    if let Err(e)=fs::write(&path,if on{"on\n"}else{"off\n"}){eprintln!("wifi: {}: {e}",path.display());}
}
/// How long startup waits for the firmware's own supplicant, and for a saved network to join.
const FIRMWARE_WAIT_MS:u64=10_000;
const JOIN_WAIT_MS:u64=20_000;
const BOOT_POLL_MS:u64=1_000;
/// Give the firmware a moment to start its supplicant; then repair it, or start one
/// (`bring_up(up)`). A repair that fails (the supplicant not ready after all) is tried again
/// shortly; one that works but leaves Wi-Fi offline once more after a while.
fn boot_repair(up:impl Fn()->bool,online:impl Fn()->bool,mut bring_up:impl FnMut(bool)->Result<()>,sleep:impl Fn(u64)){
    let mut waited=0;
    while !up() && waited<FIRMWARE_WAIT_MS{sleep(BOOT_POLL_MS);waited+=BOOT_POLL_MS;}
    for attempt in 1..=BOOT_TRIES{
        let was_up=up();
        if !was_up{eprintln!("wifi: boot: no supplicant answering; starting Wi-Fi");}
        if let Err(e)=bring_up(was_up){eprintln!("wifi: boot repair {attempt}: {e}");sleep(RETRY_MS);continue}
        let mut waited=0;
        while waited<JOIN_WAIT_MS{
            if online(){eprintln!("wifi: boot repair: online");return}
            sleep(BOOT_POLL_MS);waited+=BOOT_POLL_MS;
        }
    }
    eprintln!("wifi: boot repair: not online (no saved network in range?)");
}
const BOOT_TRIES:u32=3;
const RETRY_MS:u64=3_000;
fn helper_run(verb:&str,cancel:&AtomicBool)->Result<()>{
    let _one=HELPER.lock().unwrap_or_else(|e|e.into_inner());
    let root=card_root();
    let Some(script)=helper(&slot_power::hw("/etc/wifi/wifi_init.sh"),&root) else {return Err("Wi-Fi helper missing: System/brick_wifi.sh".into())};
    // Only slot's own helper knows "ensure" and "dhcp"; a firmware one would just print its usage.
    if matches!(verb,"ensure"|"dhcp") && !script.ends_with("System/brick_wifi.sh"){return Ok(())}
    // Inherit slot's stdout/stderr (Logs/slot.log): brick_wifi.sh moves itself to Logs/wifi.log,
    // and anything else, such as a firmware helper run directly, still leaves a trace.
    eprintln!("wifi: {} {verb}",script.display());
    let mut child=Command::new("/bin/sh").arg(&script).arg(verb).env("SLOT_ROOT",&root).stdin(Stdio::null()).spawn().map_err(|_|"Cannot start Wi-Fi helper")?;
    let start=Instant::now();loop{
        if let Some(status)=child.try_wait().map_err(|_|"Cannot check Wi-Fi helper")?{
            eprintln!("wifi: helper {verb} ended: {status}");
            return if status.success(){Ok(())}else{Err("Could not switch Wi-Fi; see Slot/Logs/wifi.log".into())}
        }
        if cancel.load(Ordering::Relaxed){let _=child.kill();let _=child.wait();eprintln!("wifi: helper {verb} cancelled");return Err("Cancelled".into())}
        if start.elapsed()>Duration::from_secs(30){let _=child.kill();let _=child.wait();eprintln!("wifi: helper {verb} timed out");return Err("Wi-Fi helper timed out; see Slot/Logs/wifi.log".into())}
        thread::sleep(Duration::from_millis(100));
    }
}
fn connect(c:&impl Supplicant,n:&Network,password:Option<String>,cancel:&AtomicBool)->Result<String>{
    let old=c.command("LIST_NETWORKS")?;
    let previous:Vec<(u32,bool,bool)>=old.lines().skip(1).filter_map(|s|{let v:Vec<_>=s.split('\t').collect();Some((v.first()?.parse().ok()?,!v.last()?.contains("[DISABLED]"),v.last()?.contains("[CURRENT]")))}).collect();
    let fresh=password.is_some() || n.saved.is_none();
    let secret=if fresh && n.security==Security::Personal{Some(psk(password.as_deref().unwrap_or(""))?)}else{None};
    let id=if fresh{c.command("ADD_NETWORK")?.parse::<u32>().map_err(|_|"Cannot create Wi-Fi profile")?}else{n.saved.unwrap()};
    let result=(||{
        if fresh{c.ok(&format!("SET_NETWORK {id} ssid {}",hex(&n.ssid)))?;c.ok(&format!("SET_NETWORK {id} key_mgmt {}",if n.security==Security::Open{"NONE"}else{"WPA-PSK"}))?;if let Some(secret)=secret{c.ok(&format!("SET_NETWORK {id} psk {secret}"))?;}}
        c.ok(&format!("SELECT_NETWORK {id}"))?;
        let deadline=Instant::now()+Duration::from_secs(20);
        while Instant::now()<deadline{
            pause(cancel,500)?;let s=c.command("STATUS")?;
            if field(&s,"id").and_then(|s|s.parse::<u32>().ok())==Some(id) && field(&s,"wpa_state")==Some("COMPLETED"){
                // Restore eligibility of previously enabled saved networks.
                for &(old,on,_) in &previous{if on{c.ok(&format!("ENABLE_NETWORK {old}"))?;}}
                // Keep the previous profile until a successful replacement has been saved.
                if c.ok("SAVE_CONFIG").is_err(){return Ok("Connected; firmware could not save profile".into())}
                if fresh{if let Some(old)=n.saved{if old!=id && c.ok(&format!("REMOVE_NETWORK {old}")).is_ok(){let _=c.ok("SAVE_CONFIG");}}}
                return Ok(if field(&s,"ip_address").is_some(){"Connected; network saved"}else{"Connected; waiting for IP address"}.into())
            }
        }Err("Connection failed. Check password / signal".into())
    })();
    if result.is_err(){
        if fresh{let _=c.ok(&format!("REMOVE_NETWORK {id}"));}
        if let Some(&(id,_,_))=previous.iter().find(|(_,_,current)|*current){let _=c.ok(&format!("SELECT_NETWORK {id}"));}else{let _=c.ok("DISCONNECT");}
        for &(id,on,_) in &previous{let _=c.ok(&format!("{}_NETWORK {id}",if on{"ENABLE"}else{"DISABLE"}));}
    }result
}
/// Scan(true) restarts Wi-Fi once if the supplicant refuses to scan.
enum Job{Open,Status,Scan(bool),Power(bool),Connect(Network,Option<String>),Disconnect}
struct Reply{on:bool,status:String,networks:Option<Vec<Network>>}
fn run(job:Job,cancel:&AtomicBool)->Result<Reply>{
    // The firmware starts a supplicant at boot with the radio still blocked: it answers, so
    // Wi-Fi looks on, but cannot scan. Repair that once when the menu opens.
    if let Job::Open=job{
        if Control::open().is_ok(){if let Err(e)=helper_run("ensure",cancel){eprintln!("wifi: ensure: {e}");}}
        return run(Job::Status,cancel)
    }
    if let Job::Power(on)=job{
        service(on,cancel)?;
        remember(on);
        if !on{return Ok(Reply{on:false,status:"Wi-Fi off".into(),networks:Some(Vec::new())})}
        for _ in 0..20{if Control::open().is_ok(){break}pause(cancel,250)?;}
        return run(Job::Scan(false),cancel)
    }
    let c=match Control::open(){Ok(c)=>c,Err(e)=>return if matches!(job,Job::Status){Ok(Reply{on:false,status:"Wi-Fi off / service unavailable".into(),networks:None})}else{Err(e)}};
    let s=c.command("STATUS")?;let connected=field(&s,"wpa_state")==Some("COMPLETED");
    let mut r=Reply{on:true,status:if connected{format!("{}  {}",name(&decode(field(&s,"ssid").unwrap_or("Connected"))),field(&s,"ip_address").unwrap_or("Waiting for IP"))}else{"Not connected".into()},networks:None};
    match job{
        Job::Status=>{},Job::Open|Job::Power(_)=>unreachable!(),
        Job::Scan(retry)=>{
            if let Err(e)=c.ok("SCAN"){
                if !retry{return Err(e)}
                // One started under a blocked radio answers but refuses scans until restarted,
                // which is what turning Wi-Fi off and on does.
                eprintln!("wifi: scan refused; restarting Wi-Fi");drop(c);service(false,cancel)?;return run(Job::Power(true),cancel)
            }
            pause(cancel,3500)?;r.networks=Some(scan_rows(&c.command("SCAN_RESULTS")?,&c.command("LIST_NETWORKS")?));if r.networks.as_ref().unwrap().is_empty(){r.status="No networks found. Try scanning again".into();}},
        Job::Connect(n,p)=>{
            r.status=connect(&c,&n,p,cancel)?;
            // Joined but no address yet: make sure a DHCP client is asking, then give the
            // router a moment, so "waiting for IP" is not the last word on a working network.
            if r.status.contains("waiting for IP"){
                if let Err(e)=helper_run("dhcp",cancel){eprintln!("wifi: dhcp: {e}");}
                let deadline=Instant::now()+Duration::from_secs(10);
                while Instant::now()<deadline{
                    pause(cancel,500)?;
                    if let Some(ip)=field(&c.command("STATUS")?,"ip_address"){r.status=format!("Connected as {ip}; network saved");break}
                }
            }
            r.networks=Some(scan_rows(&c.command("SCAN_RESULTS")?,&c.command("LIST_NETWORKS")?));
        },
        Job::Disconnect=>{c.ok("DISCONNECT")?;r.status="Disconnected".into();},
    }Ok(r)
}
struct Worker{rx:mpsc::Receiver<Result<Reply>>,cancel:Arc<AtomicBool>}
/// `typed`: when the last character went in; it shows in clear until SHOW_TYPED has passed.
/// What the keyboard is filling in. `password` holds the text, secret or not.
enum KeyTarget{Wifi(Network),RaUser,RaPassword,SsUser,SsPassword,LabelQuery,GameQuery}
struct Keyboard{target:KeyTarget,secret:bool,password:String,key:usize,page:usize,typed:Option<Instant>}
const SHOW_TYPED:Duration=Duration::from_millis(1200);
const GAME_PREVIEW_ROWS:usize=4;
/// The password as dots, but for the character just typed.
fn masked(k:&Keyboard)->String{
    let n=k.password.chars().count();
    let shown=k.typed.is_some_and(|t|t.elapsed()<SHOW_TYPED);
    if !k.secret{return k.password.chars().skip(n.saturating_sub(30)).collect()}
    let tail=n.saturating_sub(30);
    k.password.chars().enumerate().skip(tail).map(|(i,c)|if shown && i+1==n{c}else{'*'}).collect()
}
/// Three 10x4 pages, Y cycles them: QWERTY under a number row, its shifted twin (US
/// symbols over the numbers), then the rest of the symbols. The last key is space.
const KEYS:[&str;3]=[
    "1234567890qwertyuiopasdfghjkl-zxcvbnm_. ",
    "!@#$%^&*()QWERTYUIOPASDFGHJKL+ZXCVBNM?, ",
    "1234567890-_=+[]{}\\|;:'\"<>,./?`~!@#$%^& ",
];
const PAGES:[&str;3]=["abc","ABC","#!?"];
/// Rows above the network list: Wi-Fi on/off, scan, disconnect, file transfer.
const FIXED:usize=4;
/// Wi-Fi is usable for downloads: wlan0 is associated. Cheap enough to check every tick.
/// Online: wlan0 has a way out (a default route, set when DHCP hands out an address). Joined
/// to a network alone is not enough: with no address nothing can be reached.
pub fn online()->bool{fs::read_to_string(slot_power::hw("/proc/net/route")).is_ok_and(|s|routed(&s))}
fn routed(table:&str)->bool{
    table.lines().skip(1).any(|l|{let f:Vec<&str>=l.split_whitespace().collect();f.len()>1&&f[0]=="wlan0"&&f[1]=="00000000"})
}
/// The Cart Art screen: queue controls followed by unresolved attempts in this system.
/// `games`: platform, stem, label, and whether it is the Famicom label.
struct ArtList{
    games:Vec<(Platform,String,PathBuf,bool)>,
    carts:Vec<slot_store::Cart>,scopes:Vec<Option<Platform>>,scope:Option<usize>,visible:Vec<usize>,
    history:crate::art_history::History,queue:crate::art_history::Queue,
    account:crate::screenscraper::Account,account_page:bool,
}
impl ArtList{
    fn from_carts(carts:&[slot_store::Cart])->Self{
        // Only copy label metadata already loaded for the shelves. Never open ROMs or
        // enumerate archives on the UI thread just to display this menu.
        let games = carts
            .iter()
            .flat_map(|c| {
                [
                    c.label
                        .as_ref()
                        .map(|l| (c.platform, c.stem.clone(), l.clone(), false)),
                    c.famicom_label
                        .as_ref()
                        .map(|l| (c.platform, c.stem.clone(), l.clone(), true)),
                ]
            })
            .flatten()
            .collect();
        let mut carts: Vec<_> = carts
            .iter()
            .filter(|c| !matches!(c.platform, slot_store::Platform::Pico8 | slot_store::Platform::Ports))
            .cloned()
            .collect();
        carts.sort_by_cached_key(|c| (slot_store::sort_key(&c.stem), c.platform as u8));
        let mut platforms: Vec<_> = carts.iter().map(|c| c.platform).collect();
        platforms.sort_by_key(|p| *p as u8);
        platforms.dedup();
        let scopes = std::iter::once(None)
            .chain(platforms.into_iter().map(Some))
            .collect();
        let mut list = Self {
            games,
            carts,
            scopes,
            scope: None,
            visible: Vec::new(),
            history: Default::default(),
            queue: crate::art_history::Queue::New,
            account: Default::default(),
            account_page: false,
        };
        list.sort();
        list
    }
    fn select(&mut self,scope:usize){
        self.scope=Some(scope);
        self.refresh_visible();
    }
    fn scoped_carts(&self)->Vec<slot_store::Cart>{
        let scope=self.scope.and_then(|i|self.scopes[i]);
        self.carts.iter().filter(|c|scope.is_none_or(|p|p==c.platform)).cloned().collect()
    }
    fn refresh_visible(&mut self){
        let Some(scope)=self.scope else{return};
        self.visible=self.carts.iter().enumerate().filter(|(_,c)|
            self.scopes[scope].is_none_or(|p|p==c.platform) && crate::cart_art::needs_label(c)
            && matches!(self.history.status(c),Some(crate::art_history::ResultKind::Missing|crate::art_history::ResultKind::Failed))
        ).map(|(i,_)|i).collect();
    }
    fn rows(&self)->usize{if self.account_page{4}else if self.scope.is_none(){self.scopes.len()}else{self.visible.len()+3}}
    fn scope_name(&self,index:usize)->&str{self.scopes[index].map_or("ALL SYSTEMS",|p|p.dir_name())}
    fn sort(&mut self){
        self.games.sort_by_cached_key(|g|(slot_store::sort_key(&g.1),g.0 as u8,g.3));
    }
    fn fetched(&mut self,f:&Fetched){
        let Some(path)=&f.label else{return};
        let famicom=f.platform==Platform::Nes && path.parent().is_some_and(|p|p.ends_with(slot_store::FAMICOM_LABELS));
        if let Some(c)=self.carts.iter_mut().find(|c|c.platform==f.platform && c.stem==f.stem){
            if famicom{c.famicom_label=Some(path.clone())}else{c.label=Some(path.clone())}
        }
        if let Some(row)=self.games.iter_mut().find(|g|g.0==f.platform && g.1==f.stem && g.3==famicom){
            row.2=path.clone();
        }else{self.games.push((f.platform,f.stem.clone(),path.clone(),famicom));}
    }
}
/// The RetroAchievements screen: log in, test the connection, show a test unlock, log out.
/// `test`: a pretend unlock waiting for the app to toast it.
struct RaScreen{account:Option<ra::Account>,user:String,password:String,job:Option<mpsc::Receiver<RaDone>>,test:Option<TestUnlock>}
/// A pretend unlock: title, description, points, and a badge already cached on the card.
pub struct TestUnlock{pub title:String,pub description:String,pub points:u32,pub badge:Option<PathBuf>}
/// Any badge cached on the card, so the test toast looks like the real thing.
fn cached_badge(root:&Path)->Option<PathBuf>{
    let mut v:Vec<PathBuf>=fs::read_dir(root.join("System/Cache/Badges")).ok()?.filter_map(|e|Some(e.ok()?.path())).filter(|p|p.extension().is_some_and(|x|x=="png")).collect();
    v.sort();v.into_iter().next()
}
enum RaDone{LoggedIn(Result<(ra::Account,ra::Profile)>),Checked(Result<ra::Profile>)}
fn profile_line(p:&ra::Profile)->String{format!("Connected in {} ms: {}, {} points ({} softcore)",p.ms,p.user,p.score,p.softcore)}
/// The overlay behind HOME > Wi-Fi, Cart Art and RetroAchievements: `list` is set for
/// Cart Art, `ra` for RetroAchievements.
pub struct WifiMenu{root:PathBuf,transfer:Option<crate::file_transfer::Server>,transfer_progress:Option<crate::file_transfer::Progress>,rescan:Option<Vec<PathBuf>>,worker:Option<Worker>,pub face:Option<TexId>,pub dirty:bool,on:bool,status:String,networks:Vec<Network>,row:usize,keyboard:Option<Keyboard>,refreshed:Instant,art:Option<ArtJob>,fetched:Vec<Fetched>,list:Option<ArtList>,ra:Option<RaScreen>,query:Option<Option<String>>,game_index:Option<slot_store::library::SearchIndex>,game_page:usize,update:Option<crate::brick_update::UpdateScreen>}
impl WifiMenu{
    pub fn new()->Self{let mut s=Self{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:supplicant_up(),status:"Checking Wi-Fi...".into(),networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:None,query:None,game_index:None,game_page:0,update:None};s.start(Job::Open);s}
    /// The keyboard alone, to type what to search labels for: `name` to start from. Closes
    /// when done (START) or left (B); take_label_query then says which.
    pub fn label_query(name:&str)->Self{
        Self{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status:"Type part of a game's name".into(),networks:Vec::new(),row:0,
            keyboard:Some(Keyboard{target:KeyTarget::LabelQuery,secret:false,password:name.chars().take(64).collect(),key:0,page:0,typed:None}),
            refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:None,query:Some(None),game_index:None,game_page:0,update:None}
    }
    /// Local library search uses the same keyboard, without starting any network work.
    pub fn game_query(name: &str, carts: &[slot_store::Cart]) -> Self {
        let mut menu = Self::label_query(name);
        menu.keyboard.as_mut().unwrap().target = KeyTarget::GameQuery;
        menu.game_index = Some(slot_store::library::SearchIndex::new(carts));
        menu.status = "Results update as you type - no Wi-Fi needed".into();
        menu
    }
    pub fn is_game_query(&self) -> bool {
        self.keyboard.as_ref().is_some_and(|k| matches!(k.target, KeyTarget::GameQuery))
    }
    /// The preview is only text metadata. Its count and order agree with the
    /// full cartridge shelf, without scanning ROMs or loading artwork.
    fn game_preview(&self) -> Option<(usize, Vec<&slot_store::library::SearchMatch>)> {
        let keyboard = self.keyboard.as_ref()?;
        if !matches!(keyboard.target, KeyTarget::GameQuery) { return None; }
        Some(self.game_index.as_ref()?.preview(&keyboard.password, self.game_page, GAME_PREVIEW_ROWS))
    }
    /// What was typed to search labels for, once START closed the keyboard.
    pub fn take_label_query(&mut self)->Option<String>{self.query.as_mut()?.take()}
    pub fn cart_art(root:&Path,carts:&[slot_store::Cart])->Self{
        let mut list=ArtList::from_carts(carts);
        list.history=crate::art_history::History::load(root);
        list.account=crate::screenscraper::Account::load(root);
        let status="Choose a system to fetch or retry artwork".into();
        Self{root:root.to_path_buf(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status,networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:Some(list),ra:None,query:None,game_index:None,game_page:0,update:None}
    }
    pub fn update(root:&Path)->Self{
        let mut s=Self::cart_art_free();s.update=Some(crate::brick_update::UpdateScreen::open(root));s
    }
    fn cart_art_free()->Self{Self{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status:String::new(),networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:None,query:None,game_index:None,game_page:0,update:None}}
    /// The update is in: Slot should end and start again.
    pub fn take_restart(&mut self)->bool{self.update.as_mut().is_some_and(|u|u.take_restart())}
    pub fn set_battery(&mut self,battery:Option<(u8,bool)>){if let Some(u)=&mut self.update{u.battery=battery;}}
    pub fn achievements()->Self{
        let account=ra::load(&card_root());
        let status=match &account{Some(a)=>format!("Logged in as {}",a.user),None=>"Log in with your RetroAchievements account".into()};
        Self{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status,networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:Some(RaScreen{account,user:String::new(),password:String::new(),job:None,test:None}),query:None,game_index:None,game_page:0,update:None}
    }
    pub fn busy(&self)->bool{self.update.as_ref().is_some_and(|u|u.busy()) || self.worker.is_some() || self.art.is_some() || self.ra.as_ref().is_some_and(|r|r.job.is_some())}
    pub fn cancel(&mut self){
        if let Some(w)=&self.worker{w.cancel.store(true,Ordering::Relaxed);}
        if let Some(a)=&self.art{a.cancel();}
        if let Some(u)=&mut self.update{u.cancel();self.dirty=true;return}
        // A reply that comes back after this is dropped unread, so nothing gets saved.
        if let Some(r)=&mut self.ra{if r.job.take().is_some(){self.status="Cancelled".into();self.dirty=true;return}}
        if self.busy(){self.status="Cancelling...".into();self.dirty=true;}
    }
    /// Labels saved since the last call, for the library to show straight away.
    pub fn take_fetched(&mut self)->Vec<Fetched>{std::mem::take(&mut self.fetched)}
    /// The test unlock asked for since the last call, for the app to toast.
    pub fn take_test_unlock(&mut self)->Option<TestUnlock>{self.ra.as_mut()?.test.take()}
    fn poll_art(&mut self){
        let Some(job)=&self.art else{return};
        loop{match job.try_recv(){
            Ok(Progress::Step{done,total,name})=>{self.status=format!("Cart art {}/{total}: {name}",done+1);self.dirty=true;}
            Ok(Progress::Tried(attempt))=>{if let Some(l)=&mut self.list{l.history.entries.insert(attempt.key.clone(),attempt);l.refresh_visible();self.row=self.row.min(l.rows().saturating_sub(1));}self.dirty=true;},
            Ok(Progress::Got(f))=>{if let Some(l)=&mut self.list{l.fetched(&f);l.refresh_visible();self.row=self.row.min(l.rows().saturating_sub(1));}self.fetched.push(f);},
            Ok(Progress::Done(r))=>{
                self.status=match r{
                    Ok(s) if s.found+s.missing+s.failed==0=>"No games in this queue; choose a retry option".into(),
                    Ok(s)=>format!("Cart art: {} added, {} not found{}",s.found,s.missing,if s.failed>0{format!(", {} failed",s.failed)}else{String::new()}),
                    Err(e)=>e,
                };
                if let Some(l)=&mut self.list{l.sort();self.row=self.row.min(l.rows().saturating_sub(1));}
                self.art=None;self.dirty=true;self.refreshed=Instant::now();return;
            }
            Err(mpsc::TryRecvError::Empty)=>return,
            Err(mpsc::TryRecvError::Disconnected)=>{self.art=None;self.status="Cart art stopped".into();self.dirty=true;return}
        }}
    }
    fn start(&mut self,job:Job){
        if self.busy(){return}let(tx,rx)=mpsc::channel();let cancel=Arc::new(AtomicBool::new(false));let token=cancel.clone();
        if !matches!(job,Job::Status|Job::Open){self.status=match job{Job::Scan(_)=>"Scanning...",Job::Power(true)=>"Turning Wi-Fi on...",Job::Power(false)=>"Turning Wi-Fi off...",Job::Connect(..)=>"Connecting...",Job::Disconnect=>"Disconnecting...",_=>unreachable!()}.into();}
        thread::spawn(move||{let _=tx.send(run(job,&token));});self.worker=Some(Worker{rx,cancel});self.dirty=true;
    }
    pub fn at_root(root:&Path)->Self{let mut menu=Self::new();menu.root=root.to_path_buf();menu}
    #[cfg(test)]
    pub(crate) fn with_transfer(server:crate::file_transfer::Server)->Self{
        let mut menu=Self::cart_art_free();
        menu.transfer_progress=Some(server.progress());menu.transfer=Some(server);menu
    }
    pub fn transfer_open(&self)->bool{self.transfer.is_some()}
    /// Detach the running server without stopping its listener or clients.
    pub fn take_background_transfer(&mut self)->Option<crate::file_transfer::Server>{
        self.transfer_progress=None;
        self.transfer.take()
    }
    pub fn attach_background_transfer(&mut self,server:crate::file_transfer::Server){
        self.transfer_progress=Some(server.progress());
        self.transfer=Some(server);
        self.dirty=true;
    }
    /// Reopen an already-running FTP listener instantly. Starting a Wi-Fi probe
    /// here would leave an unrelated pending job after pressing B to stop FTP.
    pub fn running_transfer(root:&Path,server:crate::file_transfer::Server)->Self {
        let mut menu=Self::cart_art_free();
        menu.root=root.to_path_buf();
        menu.status="FTP server running".into();
        menu.attach_background_transfer(server);
        menu
    }
    pub fn stop_transfer(&mut self)->bool{
        let mut changed=false;
        if let Some(mut ftp)=self.transfer.take(){
            changed=ftp.stop();
            if changed {self.rescan.get_or_insert_with(Vec::new).extend(ftp.changed_paths());}
        }
        self.transfer_progress=None;
        self.dirty=true;
        changed
    }
    pub fn take_rescan(&mut self)->bool{self.rescan.take().is_some()}
    pub fn take_rescan_paths(&mut self)->Option<Vec<PathBuf>>{self.rescan.take()}
    fn open_transfer(&mut self){
        let result=crate::file_transfer::wifi_address().and_then(|ip|crate::file_transfer::Server::start_card(&self.root,(ip,crate::file_transfer::PORT).into()));
        match result{
            Ok(server)=>{self.transfer_progress=Some(server.progress());self.transfer=Some(server);},
            Err(e)=>self.status=format!("File Transfer: {e}"),
        }
        self.dirty=true;
    }
    pub fn poll(&mut self){
        if let Some(ftp)=&self.transfer{
            if self.refreshed.elapsed()<Duration::from_millis(250){return;}
            self.refreshed=Instant::now();
            let progress=ftp.progress();
            if self.transfer_progress.as_ref()!=Some(&progress){self.transfer_progress=Some(progress);self.dirty=true;}
            return;
        }
        if let Some(u)=&mut self.update{if u.poll(){self.dirty=true;}return}
        self.poll_art();
        self.poll_ra();
        if let Some(w)=&self.worker{match w.rx.try_recv(){
            Ok(r)=>{self.worker=None;match r{Ok(r)=>{self.on=r.on;self.status=r.status;if let Some(n)=r.networks{self.networks=n;}},Err(e)=>self.status=e}self.row=self.row.min(self.networks.len()+FIXED-1);self.dirty=true;self.refreshed=Instant::now();},
            Err(mpsc::TryRecvError::Disconnected)=>{self.worker=None;self.status="Wi-Fi worker stopped".into();self.dirty=true;self.refreshed=Instant::now();},_=>{}
        }}
        // Hide the character just typed once its moment is up.
        if let Some(k)=&mut self.keyboard{if k.typed.is_some_and(|t|t.elapsed()>=SHOW_TYPED){k.typed=None;self.dirty=true;}}
        // Poll only while this menu is open; scanning is always explicit. An address still on
        // its way is checked on every couple of seconds, so it shows as soon as it comes.
        let every=if self.status.contains("for IP"){Duration::from_secs(2)}else{Duration::from_secs(15)};
        if self.list.is_none() && self.ra.is_none() && !self.busy() && self.keyboard.is_none() && self.refreshed.elapsed()>every{self.start(Job::Status);}
    }
    pub fn input(&mut self,a:Action)->bool{
        if self.transfer.is_some(){
            // START returns to games while the FTP server remains alive in App.
            if matches!(a,Action::GbaDown(Btn::Start)){return true;}
            if matches!(a,Action::GbaDown(Btn::B)|Action::QuickMenu){
                let changed=self.stop_transfer();
                self.status=if changed{"File transfer stopped; checking games..."}else{"File transfer stopped"}.into();
            }
            return false;
        }
        if let Some(u)=&mut self.update{self.dirty=true;return u.input(a)}
        if self.busy(){if matches!(a,Action::GbaDown(Btn::B)){self.cancel();}return false}
        if !matches!(a,Action::GbaDown(_)|Action::ShelfLeft|Action::ShelfRight|Action::Insert){return false}
        self.dirty=true;
        if let Some(k)=&mut self.keyboard{
            match a{
                Action::GbaDown(Btn::B)=>{self.keyboard=None;if self.query.is_some(){return true}},
                Action::GbaDown(Btn::X)=>{k.password.pop();k.typed=None;self.game_page=0;},
                Action::GbaDown(Btn::Y)=>k.page=(k.page+1)%3,
                Action::GbaDown(Btn::L1) if matches!(k.target,KeyTarget::GameQuery)=>{
                    self.game_page=self.game_page.saturating_sub(1);
                },
                Action::GbaDown(Btn::R1) if matches!(k.target,KeyTarget::GameQuery)=>{
                    if let Some(index)=&self.game_index {
                        let total=index.count(&k.password);
                        if self.game_page.saturating_add(1).saturating_mul(GAME_PREVIEW_ROWS)<total {
                            self.game_page+=1;
                        }
                    }
                },
                Action::GbaDown(Btn::Up)=>k.key=(k.key+30)%40,Action::GbaDown(Btn::Down)=>k.key=(k.key+10)%40,
                Action::GbaDown(Btn::Left)|Action::ShelfLeft=>k.key=k.key/10*10+(k.key%10+9)%10,
                Action::GbaDown(Btn::Right)|Action::ShelfRight=>k.key=k.key/10*10+(k.key%10+1)%10,
                Action::GbaDown(Btn::A)|Action::Insert=>{if k.password.chars().count()<64{k.password.push(KEYS[k.page].as_bytes()[k.key] as char);k.typed=Some(Instant::now());self.game_page=0;}},
                Action::GbaDown(Btn::Start)=>{
                    if let KeyTarget::Wifi(_)=k.target{if let Err(e)=psk(&k.password){self.status=e;return false}}
                    let k=self.keyboard.take().unwrap();
                    match k.target{
                        KeyTarget::Wifi(n)=>self.start(Job::Connect(n,Some(k.password))),
                        KeyTarget::SsUser=>if let Some(l)=&mut self.list{l.account.user=k.password.trim().to_string();},
                        KeyTarget::SsPassword=>if let Some(l)=&mut self.list{l.account.password=k.password;},
                        KeyTarget::RaUser=>if let Some(ra)=&mut self.ra{ra.user=k.password.trim().to_string();},
                        KeyTarget::RaPassword=>if let Some(ra)=&mut self.ra{ra.password=k.password;},
                        KeyTarget::GameQuery=>{self.query=Some(Some(k.password.trim().to_string()));return true}
                        KeyTarget::LabelQuery=>{let q=k.password.trim().to_string();if q.is_empty(){self.keyboard=Some(k);self.status="Type part of a game's name".into();return false}self.query=Some(Some(q));return true}
                    }
                },_=>{}
            }return false
        }
        if self.list.is_some(){return self.art_input(a)}
        if self.ra.is_some(){return self.ra_input(a)}
        match a{
            Action::GbaDown(Btn::B)=>return true,
            Action::GbaDown(Btn::Up)=>self.row=(self.row+self.networks.len()+FIXED-1)%(self.networks.len()+FIXED),
            Action::GbaDown(Btn::Down)=>self.row=(self.row+1)%(self.networks.len()+FIXED),
            Action::GbaDown(Btn::A)|Action::Insert=>match self.row{0=>self.start(Job::Power(!self.on)),1=>self.start(Job::Scan(true)),2=>self.start(Job::Disconnect),3=>self.open_transfer(),_=>self.choose(false)},
            Action::GbaDown(Btn::X)=>self.choose(true),_=>{}
        }false
    }
    fn art_input(&mut self,a:Action)->bool{
        use crate::art_history::Queue;
        let list=self.list.as_mut().unwrap();let rows=list.rows();
        match a{
            Action::GbaDown(Btn::B)=>{
                if list.account_page{list.account_page=false;list.account=crate::screenscraper::Account::load(&self.root);self.row=2;self.status="Choose a queue, then fetch art".into();}
                else if let Some(scope)=list.scope.take(){self.row=scope;self.status="Choose a system to fetch or retry artwork".into();}
                else{return true}
            }
            Action::GbaDown(Btn::Up)=>{self.row=(self.row+rows-1)%rows;}
            Action::GbaDown(Btn::Down)=>{self.row=(self.row+1)%rows;}
            Action::GbaDown(Btn::Left)|Action::ShelfLeft|Action::GbaDown(Btn::Right)|Action::ShelfRight if list.scope.is_some() && !list.account_page && self.row==0=>{
                let i=Queue::ALL.iter().position(|q|*q==list.queue).unwrap();
                let delta=if matches!(a,Action::ShelfLeft|Action::GbaDown(Btn::Left)){3}else{1};list.queue=Queue::ALL[(i+delta)%4];
            }
            Action::GbaDown(Btn::A)|Action::Insert=>{
                if list.account_page{
                    match self.row{
                        0|1=>{let secret=self.row==1;self.keyboard=Some(Keyboard{target:if secret{KeyTarget::SsPassword}else{KeyTarget::SsUser},secret,password:if secret{String::new()}else{list.account.user.clone()},key:0,page:0,typed:None});self.status="Enter your ScreenScraper account details".into();}
                        2=>{if list.account.user.is_empty()||list.account.password.is_empty(){self.status="Enter your username and password first".into();}else{self.status=match list.account.save(&self.root){Ok(()) if list.account.ready()=>"Saved; fallback will use this account".into(),Ok(())=>"Saved; app API credentials still needed".into(),Err(_)=>"Cannot save ScreenScraper account".into()};}}
                        _=>{list.account.user.clear();list.account.password.clear();self.status=match list.account.save(&self.root){Ok(())=>"ScreenScraper account removed".into(),Err(_)=>"Cannot remove ScreenScraper account".into()};}
                    }
                }else if list.scope.is_none(){list.select(self.row);self.row=0;self.status="Choose a queue, then fetch art".into();}
                else if self.row==0{let i=Queue::ALL.iter().position(|q|*q==list.queue).unwrap();list.queue=Queue::ALL[(i+1)%4];}
                else if self.row==2{list.account_page=true;self.row=0;self.status=if list.account.ready(){"Automatic fallback is configured"}else if list.account.app_ready(){"Log in to enable automatic fallback"}else{"App API setup required; account can be saved"}.into();}
                else{
                    let (carts,queue)=if self.row==1{(list.scoped_carts(),list.queue)}else{
                        let c=&list.carts[list.visible[self.row-3]];
                        (vec![c.clone()],Queue::All)
                    };
                    if !online(){self.status="Connect to Wi-Fi first".into();return false}
                    self.status="Preparing selected games...".into();self.art=Some(ArtJob::selected(self.root.clone(),carts,queue));
                }
            }
            _=>{}
        }false
    }
    fn choose(&mut self,edit:bool){
        let Some(n)=self.row.checked_sub(FIXED).and_then(|i|self.networks.get(i)).cloned()else{return};
        if n.security==Security::Unsupported{self.status="Use an open or WPA/WPA2 personal network".into();return}
        if n.security==Security::Personal && (edit || n.saved.is_none()){self.status="Password is case sensitive".into();self.keyboard=Some(Keyboard{target:KeyTarget::Wifi(n),secret:true,password:String::new(),key:0,page:0,typed:None});}else{self.start(Job::Connect(n,None));}
    }
    fn poll_ra(&mut self){
        let Some(ra)=&mut self.ra else{return};
        let Some(job)=&ra.job else{return};
        let done=match job.try_recv(){Ok(d)=>d,Err(mpsc::TryRecvError::Empty)=>return,Err(mpsc::TryRecvError::Disconnected)=>{ra.job=None;self.status="RetroAchievements check stopped".into();self.dirty=true;return}};
        ra.job=None;self.dirty=true;
        self.status=match done{
            RaDone::LoggedIn(Ok((account,profile)))=>match ra::save(&card_root(),&account){
                Ok(())=>{ra.password.clear();ra.account=Some(account);self.row=1;format!("Logged in. {}",profile_line(&profile))}
                Err(e)=>format!("Logged in, but cannot save the login: {e}"),
            },
            RaDone::Checked(Ok(profile))=>profile_line(&profile),
            RaDone::LoggedIn(Err(e))|RaDone::Checked(Err(e))=>e,
        };
    }
    fn ra_input(&mut self,a:Action)->bool{
        let Some(ra)=&mut self.ra else{return true};
        let rows=if ra.account.is_some(){4}else{3};
        match a{
            Action::GbaDown(Btn::B)=>return true,
            Action::GbaDown(Btn::Up)=>self.row=(self.row+rows-1)%rows,
            Action::GbaDown(Btn::Down)=>self.row=(self.row+1)%rows,
            Action::GbaDown(Btn::A)|Action::Insert=>match (&ra.account,self.row){
                (None,0)=>self.keyboard=Some(Keyboard{target:KeyTarget::RaUser,secret:false,password:ra.user.clone(),key:0,page:0,typed:None}),
                (None,1)=>self.keyboard=Some(Keyboard{target:KeyTarget::RaPassword,secret:true,password:String::new(),key:0,page:0,typed:None}),
                (None,_)=>{
                    if ra.user.is_empty() || ra.password.is_empty(){self.status="Enter your username and password first".into();return false}
                    if !online(){self.status="Connect to Wi-Fi first".into();return false}
                    let (user,password)=(ra.user.clone(),ra.password.clone());let(tx,rx)=mpsc::channel();
                    thread::spawn(move||{let _=tx.send(RaDone::LoggedIn(ra::login(&user,&password)));});
                    ra.job=Some(rx);self.status="Logging in...".into();
                }
                (Some(_),0)=>{}
                (Some(account),1)=>{
                    if !online(){self.status="Connect to Wi-Fi first".into();return false}
                    let account=account.clone();let(tx,rx)=mpsc::channel();
                    thread::spawn(move||{let _=tx.send(RaDone::Checked(ra::check(&account)));});
                    ra.job=Some(rx);self.status="Testing the connection...".into();
                }
                (Some(_),2)=>{
                    ra.test=Some(TestUnlock{title:"Test Achievement".into(),description:"Toasts are working".into(),points:5,badge:cached_badge(&card_root())});
                    self.status="Test achievement sent".into();
                }
                (Some(_),_)=>{
                    match ra::forget(&card_root()){
                        Ok(())=>{ra.account=None;self.row=0;self.status="Logged out".into();}
                        Err(e)=>self.status=format!("Cannot log out: {e}"),
                    }
                }
            },
            _=>{}
        }false
    }
    fn render_ra(&self,ra:&RaScreen)->UndoFace{
        let mut p=Painter::new();p.text("RETROACHIEVEMENTS",32,24,576,30.0);p.text(&self.status,32,66,576,18.0);
        let rows:Vec<String>=match &ra.account{
            None=>vec![format!("USERNAME     {}",if ra.user.is_empty(){"-"}else{&ra.user}),format!("PASSWORD     {}",if ra.password.is_empty(){"-".to_string()}else{"*".repeat(ra.password.chars().count().min(20))}),"LOG IN".into()],
            Some(a)=>vec![format!("LOGGED IN AS  {}",a.user),"TEST CONNECTION".into(),"TEST ACHIEVEMENT".into(),"LOG OUT".into()],
        };
        for (i,label) in rows.iter().enumerate(){let y=110+i*41;if i==self.row{p.rect(0,y,640,38,slot_ui::edge());}p.text(label,32,y+1,576,23.0);}
        p.text(if self.busy(){"B CANCEL"}else{"B BACK    A SELECT"},24,430,592,20.0);
        UndoFace{rgba:p.rgba,w:640,h:480}
    }
    fn render_art(&self,list:&ArtList)->UndoFace{
        let mut p=Painter::new();
        let title=if list.account_page{"SCREENSCRAPER".into()}else{list.scope.map_or("CART ART".into(),|i|format!("CART ART / {}",list.scope_name(i)))};
        p.text(&title,32,24,576,30.0);p.text(&self.status,24,66,592,17.0);
        let rows=list.rows();let first=self.row.saturating_sub(6);
        for (line,row) in (first..rows.min(first+7)).enumerate(){let y=110+line*41;if row==self.row{p.rect(0,y,640,38,slot_ui::edge());}
            let label=if list.account_page{match row{0=>format!("USERNAME   {}",if list.account.user.is_empty(){"-"}else{&list.account.user}),1=>format!("PASSWORD   {}",if list.account.password.is_empty(){"-"}else{"********"}),2=>"SAVE ACCOUNT".into(),_=>"REMOVE ACCOUNT".into()}}
            else if list.scope.is_none(){let scope=list.scopes[row];let total=list.carts.iter().filter(|c|scope.is_none_or(|s|s==c.platform)).count();format!("{}     {total} games",list.scope_name(row))}
            else{match row{0=>format!("QUEUE    < {} >",list.queue.label()),1=>"FETCH ART FOR THIS QUEUE".into(),2=>"SCREENSCRAPER ACCOUNT".into(),_=>{
                let at=list.visible[row-3];let c=&list.carts[at];let status=list.history.status(c).map_or("FAILED",|r|r.label());
                format!("{status} / {}  {}",c.platform.dir_name(),c.stem)
            }}};
            p.text(&label,24,y+1,592,21.0);
        }
        if list.scope.is_some() && !list.account_page && list.visible.is_empty(){p.text("No failed or unmatched games",24,250,592,21.0);}
        p.text(if self.busy(){"B CANCEL"}else if list.scope.is_some() && !list.account_page && self.row>=3{"B BACK    A RETRY"}else{"B BACK    A SELECT"},24,430,592,20.0);
        UndoFace{rgba:p.rgba,w:640,h:480}
    }
    fn render_update(&self,u:&crate::brick_update::UpdateScreen)->UndoFace{
        let mut p=Painter::new();p.text("UPDATE",32,24,576,30.0);p.text(&u.status,32,66,576,18.0);
        let rows=u.lines();
        for (i,label) in rows.iter().enumerate(){let y=110+i*41;if i==u.row(){p.rect(0,y,640,38,slot_ui::edge());}p.text(label,32,y+1,576,23.0);}
        let (have,notes)=u.about();let mut y=110+rows.len()*41+10;
        p.text(&have,32,y,576,18.0);y+=34;
        let lines:Vec<String>=notes.iter().flat_map(|n|wrap(n,576,17.0)).collect();
        let room=(420-y)/24;let max=lines.len().saturating_sub(room);u.scroll_max.set(max);
        for (i,line) in lines.iter().skip(u.scroll()).take(room).enumerate(){p.text(line,32,y+i*24,576,17.0);}
        let legend=if max>0 && !u.legend().is_empty(){format!("{}    L1/R1 SCROLL",u.legend())}else{u.legend().to_string()};
        p.text(&legend,24,430,592,20.0);
        UndoFace{rgba:p.rgba,w:640,h:480}
    }
    fn render_transfer(&self,ftp:&crate::file_transfer::Server)->UndoFace{
        let mut p=Painter::new();
        p.text("FILE TRANSFER  /  SERVER RUNNING",32,24,576,26.0);
        p.text("Keep connected on trusted Wi-Fi; FTP is not encrypted",24,68,592,17.0);
        p.text(&format!("HOST  {}",ftp.address().ip()),24,124,592,28.0);
        p.text(&format!("PORT  {}     USER  {}",ftp.address().port(),crate::file_transfer::USER),24,167,592,23.0);
        p.text(&format!("PASSWORD  {}",ftp.password()),24,210,592,28.0);
        p.text("SD CARD ROOT  /   FTP passive mode",24,254,592,19.0);
        if let Some(progress)=&self.transfer_progress{
            p.text(&format!("{} connected   Up {:.1} MB   Down {:.1} MB",progress.clients,progress.uploaded as f64/1048576.0,progress.downloaded as f64/1048576.0),24,304,592,18.0);
            p.text(&progress.activity,24,344,592,19.0);
        }
        p.text("START: BROWSE WITH FTP RUNNING    POWER: STOP",24,386,592,15.0);
        p.text("B STOP SERVER        START RUN IN BACKGROUND",24,426,592,17.0);
        UndoFace{rgba:p.rgba,w:640,h:480}
    }
    /// Compact keyboard leaves room for live counts and four matching titles.
    /// Shoulder buttons page through the preview; START still opens the full shelf.
    fn render_game_keyboard(&self,p:&mut Painter,k:&Keyboard){
        let (total, matches)=self.game_preview().unwrap_or((0,Vec::new()));
        p.text("SEARCH FOR",24,103,592,24.0);
        let shown=if k.password.is_empty(){"Type a game name".to_string()}else{masked(k)};
        p.text(&format!("{}   ({}/64)",shown,k.password.chars().count()),24,141,592,21.0);
        p.text(&format!("{} {}  -  LIVE RESULTS",total,if total==1{"MATCH"}else{"MATCHES"}),24,169,592,16.0);

        // Keep the same 10x4 key map, just narrower on the search screen.
        for i in 0..40 {
            let x=20+(i%10)*35;
            let y=202+(i/10)*43;
            if i==k.key{p.rect(x,y,33,40,slot_ui::edge());}
            let letter=&KEYS[k.page][i..i+1];
            if letter==" "{p.text("SPC",x,y+5,33,13.0);}
            else{p.text(letter,x,y+3,33,21.0);}
        }

        p.text("MATCHING GAMES",385,197,230,17.0);
        if matches.is_empty(){
            p.text("NO MATCHES",385,261,230,18.0);
            p.text("Try fewer letters",385,294,230,14.0);
        }else{
            for (i,entry) in matches.iter().enumerate(){
                p.text(&format!("{}  {}",entry.platform.dir_name(),entry.title),384,219+i*34,233,16.0);
            }
            let pages=total.div_ceil(GAME_PREVIEW_ROWS);
            p.text(&format!("PAGE {} / {}   L1 / R1",self.game_page+1,pages),385,359,230,14.0);
        }
        p.text("A TYPE   X DELETE   Y KEYS   L1/R1 RESULTS",20,390,600,16.0);
        p.text("B CANCEL                START BROWSE",24,430,592,20.0);
    }
    pub fn render(&self)->UndoFace{
        if let Some(ftp)=&self.transfer{return self.render_transfer(ftp)}
        if let Some(u)=&self.update{return self.render_update(u)}
        if self.keyboard.is_none(){
            if let Some(list)=&self.list{return self.render_art(list)}
            if let Some(ra)=&self.ra{return self.render_ra(ra)}
        }
        let mut p=Painter::new();p.text(if self.list.is_some(){"SCREENSCRAPER"}else if self.ra.is_some(){"RETROACHIEVEMENTS"}else if self.is_game_query(){"SEARCH GAMES"}else if self.query.is_some(){"FIND A LABEL"}else{"WI-FI"},32,24,576,30.0);p.text(&self.status,32,66,576,18.0);
        if let Some(k)=&self.keyboard{
            if matches!(k.target,KeyTarget::GameQuery){
                self.render_game_keyboard(&mut p,k);
            }else{
            let title=match &k.target{KeyTarget::Wifi(n)=>name(&n.ssid),KeyTarget::RaUser|KeyTarget::SsUser=>"USERNAME".into(),KeyTarget::RaPassword|KeyTarget::SsPassword=>"PASSWORD".into(),KeyTarget::LabelQuery|KeyTarget::GameQuery=>"SEARCH FOR".into()};
            p.text(&title,32,102,576,24.0);p.text(&format!("{}  ({}/64)",masked(k),k.password.len()),32,142,576,21.0);
            for i in 0..40{let x=32+i%10*58;let y=196+i/10*45;if i==k.key{p.rect(x,y,54,41,slot_ui::edge());}let c=&KEYS[k.page][i..i+1];if c==" "{p.text("SPC",x,y+9,54,17.0);}else{p.text(c,x,y+3,54,26.0);}}
            p.text(&format!("A TYPE    X DELETE    Y {}",PAGES.iter().enumerate().map(|(i,n)|if i==k.page{format!("[{n}]")}else{n.to_string()}).collect::<Vec<_>>().join(" / ")),24,392,592,18.0);p.text(match k.target{KeyTarget::Wifi(_)=>"B BACK                    START CONNECT",KeyTarget::LabelQuery|KeyTarget::GameQuery=>"B BACK                    START SEARCH",_=>"B BACK                    START DONE"},24,430,592,20.0);
            }
        }else{
            let first=if self.row<7{0}else{self.row-6};
            for (line,row) in (first..(self.networks.len()+FIXED).min(first+7)).enumerate(){let y=110+line*41;if row==self.row{p.rect(0,y,640,38,slot_ui::edge());}
                let label=match row{0=>format!("WI-FI                          {}",if self.on{"ON"}else{"OFF"}),1=>"SCAN NETWORKS".into(),2=>"DISCONNECT".into(),3=>"FILE TRANSFER".into(),_=>{let n=&self.networks[row-FIXED];format!("{}  {}  {} dBm",name(&n.ssid),if n.saved.is_some(){"[SAVED]"}else if n.security==Security::Open{"[OPEN]"}else{"[LOCK]"},n.signal)}};p.text(&label,32,y+1,576,23.0);
            }p.text(if self.busy(){"B CANCEL"}else{"B BACK    A SELECT    X EDIT PASSWORD"},24,430,592,20.0);
        }UndoFace{rgba:p.rgba,w:640,h:480}
    }
}
/// `text` in lines no wider than `w` at `px`, broken between words.
fn wrap(text:&str,w:usize,px:f32)->Vec<String>{
    let Some(font)=slot_ui::text::label_font()else{return vec![text.to_string()]};
    let mut lines=Vec::new();let mut line=String::new();
    for word in text.split_whitespace(){
        let next=if line.is_empty(){word.to_string()}else{format!("{line} {word}")};
        if !line.is_empty() && slot_ui::text::line_width(font,&next,px,0.0)>w as f32{lines.push(std::mem::replace(&mut line,word.to_string()));}else{line=next;}
    }
    if !line.is_empty(){lines.push(line);}lines
}
struct Painter{rgba:Vec<u8>}
impl Painter{
    fn new()->Self{let mut s=Self{rgba:vec![0;640*480*4]};s.rect(0,0,640,480,slot_ui::opening());s}
    fn rect(&mut self,x:usize,y:usize,w:usize,h:usize,c:[f32;4]){let c=c.map(|v|(v*255.0)as u8);for yy in y..(y+h).min(480){for xx in x..(x+w).min(640){self.rgba[(yy*640+xx)*4..(yy*640+xx)*4+4].copy_from_slice(&c)}}}
    fn text(&mut self,s:&str,x:usize,y:usize,w:usize,px:f32){let Some(font)=slot_ui::text::label_font()else{return};let mut s=s.to_string();while slot_ui::text::line_width(font,&s,px,0.0)>w as f32 && !s.is_empty(){s.pop();}
        let h=40;let mut face=vec![0;w*h*4];let layout=slot_ui::text::Layout{lines:vec![s],px,tracking:0.0};slot_ui::text::draw_centred(&mut face,w as u32,h as u32,&layout,[238,238,242]);
        for yy in 0..h.min(480-y){for xx in 0..w.min(640-x){let a=(yy*w+xx)*4;let b=((y+yy)*640+x+xx)*4;let alpha=face[a+3]as u32;for ch in 0..3{self.rgba[b+ch]=((face[a+ch]as u32*alpha+self.rgba[b+ch]as u32*(255-alpha))/255)as u8;}}}
    }
}
#[cfg(test)]mod tests{
    use super::*;use std::cell::{Cell,RefCell};
    fn art_cart(platform:Platform,stem:&str,label:Option<PathBuf>)->slot_store::Cart{
        slot_store::Cart{platform,stem:stem.into(),rom:PathBuf::from(format!("unavailable/{stem}.zip")),label,famicom_label:None,title:String::new(),code:String::new(),shell:None}
    }
    #[test]fn cart_art_opens_from_cached_metadata_even_when_roms_are_unavailable(){
        let d=tempfile::tempdir().unwrap();
        let carts:Vec<_>=(0..20_000).map(|i|art_cart(Platform::Gba,&format!("Game {i:05}"),(i%2==0).then(||d.path().join(format!("Labels/GBA/Game {i:05}.png"))))).collect();
        let start=Instant::now();
        let mut menu=WifiMenu::cart_art(d.path(),&carts);
        eprintln!("Cart Art: 20,000 cached games in {:?}",start.elapsed());
        assert_eq!(menu.list.as_ref().unwrap().games.len(),10_000);
        assert_eq!(menu.list.as_ref().unwrap().games[0].1,"Game 00000");
        assert!(!menu.busy());
        assert!(menu.input(Action::GbaDown(Btn::B)),"opening the menu must not start a scan or download");
    }
    #[test]fn downloaded_labels_update_rows_without_a_rescan_or_losing_famicom_art(){
        let mut nes=art_cart(Platform::Nes,"Zelda",Some("Labels/NES/Zelda.png".into()));
        nes.famicom_label=Some("Labels/NES/Famicom/Zelda.png".into());
        let mut list=ArtList::from_carts(&[nes,art_cart(Platform::Gg,"Sonic",None)]);
        for (platform,stem,path) in [(Platform::Gg,"Sonic","Labels/GG/Sonic.png"),(Platform::Nes,"Zelda","Labels/NES/Famicom/Zelda.png"),(Platform::Sms,"Sonic","Labels/SMS/Sonic.png")]{
            list.fetched(&Fetched{platform,stem:stem.into(),label:Some(path.into())});
        }
        list.sort();
        assert_eq!(list.games.iter().map(|g|(g.0,g.1.as_str(),g.3)).collect::<Vec<_>>(),[
            (Platform::Gg,"Sonic",false),(Platform::Sms,"Sonic",false),
            (Platform::Nes,"Zelda",false),(Platform::Nes,"Zelda",true),
        ]);
    }
    #[test]fn finishing_art_downloads_preserves_cached_rows_without_scanning_roms(){
        let d=tempfile::tempdir().unwrap();
        let mut menu=WifiMenu::cart_art(d.path(),&[art_cart(Platform::Ps1,"Crash",Some(d.path().join("Labels/PS1/Crash.png")))]);
        // The worker has an empty card; the displayed metadata deliberately has a label.
        // A full rescan on Done would erase that row. No network is needed for zero games.
        menu.art=Some(ArtJob::start(d.path().to_path_buf()));
        let start=Instant::now();
        while menu.art.is_some(){
            assert!(start.elapsed()<Duration::from_secs(3));
            menu.poll_art();thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(menu.list.as_ref().unwrap().games[0].1,"Crash");
        assert!(menu.input(Action::GbaDown(Btn::B)));
    }
    #[test]fn art_system_scope_and_account_keyboard_work_without_scanning(){
        let d=tempfile::tempdir().unwrap();let mut m=WifiMenu::cart_art(d.path(),&[art_cart(Platform::Sms,"Alex",None),art_cart(Platform::Gg,"Sonic",None)]);
        let sms=m.list.as_ref().unwrap().scopes.iter().position(|p|*p==Some(Platform::Sms)).unwrap();m.row=sms;
        m.input(Action::GbaDown(Btn::A));let l=m.list.as_ref().unwrap();assert!(l.visible.is_empty());assert_eq!(l.scoped_carts().len(),1);assert_eq!(l.scoped_carts()[0].platform,Platform::Sms);
        m.input(Action::ShelfRight);assert_eq!(m.list.as_ref().unwrap().queue,crate::art_history::Queue::Failed);
        if let Ok(dir)=std::env::var("SLOT_ART_PREVIEW_DIR"){
            fs::create_dir_all(&dir).unwrap();
            for (name,scope,row) in [("systems",None,0),("sms",Some(sms),0)]{m.list.as_mut().unwrap().scope=scope;m.row=row;let f=m.render();let mut e=png::Encoder::new(fs::File::create(Path::new(&dir).join(format!("{name}.png"))).unwrap(),640,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();}
        }
        m.row=2;m.input(Action::GbaDown(Btn::A));assert!(m.list.as_ref().unwrap().account_page);
        m.input(Action::GbaDown(Btn::A));m.keyboard.as_mut().unwrap().password="player".into();m.input(Action::GbaDown(Btn::Start));
        m.row=1;m.input(Action::GbaDown(Btn::A));m.keyboard.as_mut().unwrap().password="secret".into();assert_eq!(masked(m.keyboard.as_ref().unwrap()),"******");m.input(Action::GbaDown(Btn::Start));
        m.row=2;m.input(Action::GbaDown(Btn::A));let account=crate::screenscraper::Account::load(d.path());assert_eq!(account.user,"player");assert_eq!(account.password,"secret");assert!(!account.ready());assert!(m.status.contains("app API"));
        m.row=3;m.input(Action::GbaDown(Btn::A));assert!(crate::screenscraper::Account::load(d.path()).user.is_empty());
    }
    struct Mock{steps:RefCell<std::collections::VecDeque<(String,Result<String>)>>}
    impl Mock{fn new(steps:Vec<(&str,Result<&str>)>)->Self{Self{steps:RefCell::new(steps.into_iter().map(|(a,b)|(a.into(),b.map(String::from))).collect())}}fn done(&self){assert!(self.steps.borrow().is_empty());}}
    impl Supplicant for Mock{fn command(&self,s:&str)->Result<String>{let(a,b)=self.steps.borrow_mut().pop_front().expect("unexpected command");assert_eq!(a,s);b}}
    fn network()->Network{Network{ssid:b"Test".to_vec(),signal:-40,security:Security::Personal,saved:None}}
    #[test]fn boot_repair_repairs_a_late_firmware_supplicant(){
        let t=Cell::new(0u64);let runs=RefCell::new(Vec::new());
        // The firmware's supplicant appears 5 s in; a network joins 3 s after the repair.
        boot_repair(||t.get()>=5_000,||!runs.borrow().is_empty()&&t.get()>=8_000,|up|{runs.borrow_mut().push(up);Ok(())},|ms|t.set(t.get()+ms));
        assert_eq!(*runs.borrow(),vec![true],"repaired the firmware's supplicant once");
    }
    #[test]fn boot_repair_starts_wifi_itself_when_the_firmware_never_does(){
        let t=Cell::new(0u64);let started=Cell::new(false);
        boot_repair(||started.get(),||started.get()&&t.get()>=FIRMWARE_WAIT_MS+2_000,|up|{assert!(!up);started.set(true);Ok(())},|ms|t.set(t.get()+ms));
        assert!(started.get(),"Wi-Fi was never started");
        assert!(t.get()<=FIRMWARE_WAIT_MS+JOIN_WAIT_MS,"waited past joining");
    }
    #[test]fn boot_repair_gives_up_after_its_tries(){
        let t=Cell::new(0u64);let runs=Cell::new(0);
        boot_repair(||true,||false,|_|{runs.set(runs.get()+1);Ok(())},|ms|t.set(t.get()+ms));
        assert_eq!(runs.get(),BOOT_TRIES);
    }
    #[test]fn a_failed_repair_is_tried_again_soon(){
        // As on the Brick: the first repair finds the supplicant not answering yet.
        let t=Cell::new(0u64);let runs=RefCell::new(Vec::new());let fixed=Cell::new(false);
        boot_repair(||true,||fixed.get(),|_|{runs.borrow_mut().push(t.get());if runs.borrow().len()==1{Err("no supplicant answering".into())}else{fixed.set(true);Ok(())}},|ms|t.set(t.get()+ms));
        let runs=runs.borrow();
        assert_eq!(runs.len(),2);
        assert!(runs[1]-runs[0]<=RETRY_MS,"waited {} ms to try again",runs[1]-runs[0]);
    }
    #[test]fn joined_without_an_address_is_not_online(){
        let head="Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT\n";
        assert!(!routed(head));
        assert!(!routed(&format!("{head}wlan0\t0001A8C0\t00000000\t0001\t0\t0\t0\t00FFFFFF\t0\t0\t0\n")),"a local route only");
        assert!(routed(&format!("{head}wlan0\t00000000\t0101A8C0\t0003\t0\t0\t0\t00000000\t0\t0\t0\n")));
    }
    #[test]fn wifi_comes_back_as_it_was_left(){
        let d=tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("Config")).unwrap();
        fs::write(d.path().join(CHOICE_FILE),"off\n").unwrap();assert!(!wanted(d.path()));
        fs::write(d.path().join(CHOICE_FILE),"on\n").unwrap();assert!(wanted(d.path()));
        fs::remove_file(d.path().join(CHOICE_FILE)).unwrap();
        fs::write(d.path().join("Config/wpa_supplicant.conf"),"ctrl_interface=/var/run/wpa_supplicant\nnetwork={\n ssid=\"Home\"\n}\n").unwrap();
        assert!(wanted(d.path()),"a saved network and no choice yet: Wi-Fi on");
    }
    #[test]
    fn game_keyboard_updates_match_counts_immediately_without_touching_roms() {
        let carts=[
            art_cart(Platform::Md,"Sonic",None),
            art_cart(Platform::Gg,"Sonic",None),
            art_cart(Platform::Sms,"Sonic 2",None),
            art_cart(Platform::Gba,"Mario",None),
        ];
        let mut menu=WifiMenu::game_query("",&carts);
        assert_eq!(menu.game_preview().unwrap().0,4);
        let s=KEYS[0].find('s').unwrap();
        menu.keyboard.as_mut().unwrap().key=s;
        assert!(!menu.input(Action::GbaDown(Btn::A)));
        let (count,shown)=menu.game_preview().unwrap();
        assert_eq!(count,3);
        assert_eq!(shown.len(),3);
        assert!(shown.iter().all(|hit|hit.stem.starts_with("Sonic")));
        assert!(menu.dirty);
        assert_eq!(menu.render().rgba.len(),640*480*4);
        assert!(!menu.input(Action::GbaDown(Btn::X)));
        assert_eq!(menu.game_preview().unwrap().0,4);
        assert!(menu.input(Action::GbaDown(Btn::B)),"cancel should close search");
        assert_eq!(menu.take_label_query(),None,"cancel must not apply the partial query");
    }

    #[test]
    fn game_preview_shoulder_paging_and_start_keep_existing_browse_behavior() {
        let carts:Vec<_>=(0..10).map(|i|art_cart(Platform::Gba,&format!("Game {i:02}"),None)).collect();
        let mut menu=WifiMenu::game_query("game",&carts);
        assert_eq!(menu.game_preview().unwrap().1[0].stem,"Game 00");
        assert!(!menu.input(Action::GbaDown(Btn::R1)));
        assert_eq!(menu.game_page,1);
        assert_eq!(menu.game_preview().unwrap().1[0].stem,"Game 04");
        menu.input(Action::GbaDown(Btn::R1));
        assert_eq!(menu.game_page,2);
        menu.input(Action::GbaDown(Btn::R1));
        assert_eq!(menu.game_page,2,"cannot paginate beyond last match");
        menu.input(Action::GbaDown(Btn::L1));
        assert_eq!(menu.game_page,1);
        menu.input(Action::GbaDown(Btn::X));
        assert_eq!(menu.game_page,0,"editing restarts from first results");
        assert!(menu.input(Action::GbaDown(Btn::Start)));
        assert_eq!(menu.take_label_query(),Some("gam".into()));
    }

    #[test]fn the_label_keyboard_starts_from_the_name_and_hands_back_what_was_typed(){
        let mut m=WifiMenu::label_query("Yoshi");
        assert!(!m.input(Action::GbaDown(Btn::X)),"deleting closed it");
        assert!(m.input(Action::GbaDown(Btn::Start)),"START did not close it");
        assert_eq!(m.take_label_query(),Some("Yosh".into()));
        assert_eq!(m.take_label_query(),None);
        let mut m=WifiMenu::label_query("Yoshi");
        assert!(m.input(Action::GbaDown(Btn::B)),"B did not close it");
        assert_eq!(m.take_label_query(),None,"B searched");
        let mut m=WifiMenu::label_query("");
        assert!(!m.input(Action::GbaDown(Btn::Start)),"an empty search closed it");
    }
    #[test]fn parse_scans(){let n=scan_rows("header\na\t2412\t-80\t[WPA2-PSK-CCMP]\tMy\\x20WiFi\nb\t2412\t-40\t[WPA2-PSK-CCMP]\tMy WiFi\nc\t2412\t-30\t[ESS]\tCafe\nd\t2412\t-20\t[WPA2-EAP-CCMP]\tWork\n","header\n7\tMy WiFi\tany\t[CURRENT]\n");assert_eq!(n.len(),3);assert_eq!(n[0].security,Security::Unsupported);assert_eq!(n[2].signal,-40);assert_eq!(n[2].saved,Some(7));}
    #[test]fn password_bytes(){assert!(psk("short").is_err());assert!(psk("1234567\n").is_err());assert_eq!(psk("Abc\"\\123").unwrap(),"\"Abc\"\\123\"");assert_eq!(psk(&"a".repeat(64)).unwrap(),"a".repeat(64));assert!(psk(&"z".repeat(64)).is_err());}
    #[test]fn helper_prefers_card_then_firmware(){
        let d=tempfile::tempdir().unwrap();let fw=d.path().join("wifi_init.sh");let root=d.path().join("Slot");
        assert_eq!(helper(&fw,&root),None);
        fs::write(&fw,"").unwrap();assert_eq!(helper(&fw,&root),Some(fw.clone()));
        fs::create_dir_all(root.join("System")).unwrap();fs::write(root.join("System/brick_wifi.sh"),"").unwrap();
        assert_eq!(helper(&fw,&root),Some(root.join("System/brick_wifi.sh")));
    }
    #[test]fn all_keys(){for s in KEYS{assert_eq!(s.len(),40);assert_eq!(s.as_bytes()[39],b' ',"space last on every page");}for b in 32..=126{assert!(KEYS.iter().any(|s|s.as_bytes().contains(&b)),"{b}");}}
    #[test]fn the_last_typed_character_shows_briefly(){
        let mut k=Keyboard{target:KeyTarget::Wifi(network()),secret:true,password:"hunter2".into(),key:0,page:0,typed:Some(Instant::now())};
        assert_eq!(masked(&k),"******2");
        k.typed=Some(Instant::now()-SHOW_TYPED);assert_eq!(masked(&k),"*******");
        k.typed=None;assert_eq!(masked(&k),"*******");
        k.password="x".repeat(40)+"y";k.typed=Some(Instant::now());
        let m=masked(&k);assert_eq!(m.chars().count(),30);assert!(m.ends_with("*y"));
    }
    #[test]fn a_username_is_typed_in_clear(){
        let k=Keyboard{target:KeyTarget::RaUser,secret:false,password:"Benni".into(),key:0,page:0,typed:None};
        assert_eq!(masked(&k),"Benni");
    }
    #[test]fn ra_screen_types_the_login_and_needs_both_fields(){
        let mut m=WifiMenu{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status:String::new(),networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:Some(RaScreen{account:None,user:String::new(),password:String::new(),job:None,test:None}),query:None,game_index:None,game_page:0,update:None};
        m.input(Action::GbaDown(Btn::A));assert!(m.keyboard.as_ref().is_some_and(|k|!k.secret));
        // q w e on the letter row
        m.input(Action::GbaDown(Btn::Down));for _ in 0..3{m.input(Action::GbaDown(Btn::A));m.input(Action::GbaDown(Btn::Right));}
        m.input(Action::GbaDown(Btn::Start));assert_eq!(m.ra.as_ref().unwrap().user,"qwe");assert!(m.keyboard.is_none());
        m.input(Action::GbaDown(Btn::Down));m.input(Action::GbaDown(Btn::Down));m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.status,"Enter your username and password first");assert!(!m.busy());
        m.input(Action::GbaDown(Btn::Up));m.input(Action::GbaDown(Btn::A));assert!(m.keyboard.as_ref().is_some_and(|k|k.secret));
        if let Ok(path)=std::env::var("SLOT_RA_PREVIEW"){m.keyboard=None;m.ra.as_mut().unwrap().password="secret".into();let f=m.render();let mut e=png::Encoder::new(fs::File::create(path).unwrap(),640,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();}
        assert!(!m.input(Action::GbaDown(Btn::B)),"B closes the keyboard first");assert!(m.keyboard.is_none());
        assert!(m.input(Action::GbaDown(Btn::B)),"then the screen");
    }
    #[test]fn test_achievement_queues_a_pretend_unlock_with_a_cached_badge(){
        let dir=std::env::temp_dir().join(format!("slot-ra-test-{}",std::process::id()));let _=fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("System/Cache/Badges")).unwrap();fs::write(dir.join("System/Cache/Badges/b.png"),b"").unwrap();fs::write(dir.join("System/Cache/Badges/a.png"),b"").unwrap();
        assert_eq!(cached_badge(&dir),Some(dir.join("System/Cache/Badges/a.png")));
        assert_eq!(cached_badge(&dir.join("nowhere")),None);let _=fs::remove_dir_all(&dir);
        let account=ra::Account{user:"Benni".into(),token:"t".into()};
        let mut m=WifiMenu{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status:String::new(),networks:Vec::new(),row:0,keyboard:None,refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:Some(RaScreen{account:Some(account),user:String::new(),password:String::new(),job:None,test:None}),query:None,game_index:None,game_page:0,update:None};
        m.input(Action::GbaDown(Btn::Up));assert_eq!(m.row,3,"four rows when logged in, wrapping to LOG OUT");
        m.input(Action::GbaDown(Btn::Up));assert_eq!(m.row,2);
        assert!(m.take_test_unlock().is_none());
        m.input(Action::GbaDown(Btn::A));
        let t=m.take_test_unlock().unwrap();assert_eq!((t.title.as_str(),t.points),("Test Achievement",5));
        assert!(m.take_test_unlock().is_none(),"taken once");assert!(!m.busy());
    }
    #[test]fn qwerty_under_a_number_row(){
        let rows=|p:usize|KEYS[p].as_bytes().chunks(10).map(|r|String::from_utf8_lossy(r).into_owned()).collect::<Vec<_>>();
        assert_eq!(rows(0),["1234567890","qwertyuiop","asdfghjkl-","zxcvbnm_. "]);
        assert_eq!(rows(1)[1..3],["QWERTYUIOP".to_string(),"ASDFGHJKL+".to_string()]);
        for i in 10..40{let (l,u)=(KEYS[0].as_bytes()[i],KEYS[1].as_bytes()[i]);if l.is_ascii_lowercase(){assert_eq!(u,l.to_ascii_uppercase(),"shift keeps letters in place");}}
    }
    #[test]fn failed_setup_restores_previous_connection(){let m=Mock::new(vec![("LIST_NETWORKS",Ok("header\n3\tOld\tany\t[CURRENT]\n4\tOther\tany\t[DISABLED]")),("ADD_NETWORK",Ok("8")),("SET_NETWORK 8 ssid 54657374",Err("rejected".into())),("REMOVE_NETWORK 8",Ok("OK")),("SELECT_NETWORK 3",Ok("OK")),("ENABLE_NETWORK 3",Ok("OK")),("DISABLE_NETWORK 4",Ok("OK"))]);assert!(connect(&m,&network(),Some("password".into()),&AtomicBool::new(false)).is_err());m.done();}
    #[test]fn cancelled_connect_removes_unsaved_profile(){let m=Mock::new(vec![("LIST_NETWORKS",Ok("header")),("ADD_NETWORK",Ok("8")),("SET_NETWORK 8 ssid 54657374",Ok("OK")),("SET_NETWORK 8 key_mgmt WPA-PSK",Ok("OK")),("SET_NETWORK 8 psk \"password\"",Ok("OK")),("SELECT_NETWORK 8",Ok("OK")),("REMOVE_NETWORK 8",Ok("OK")),("DISCONNECT",Ok("OK"))]);assert_eq!(connect(&m,&network(),Some("password".into()),&AtomicBool::new(true)).unwrap_err(),"Cancelled");m.done();}
    #[test]fn successful_connect_saves_only_after_association(){let m=Mock::new(vec![("LIST_NETWORKS",Ok("header")),("ADD_NETWORK",Ok("8")),("SET_NETWORK 8 ssid 54657374",Ok("OK")),("SET_NETWORK 8 key_mgmt WPA-PSK",Ok("OK")),("SET_NETWORK 8 psk \"password\"",Ok("OK")),("SELECT_NETWORK 8",Ok("OK")),("STATUS",Ok("id=8\nwpa_state=COMPLETED\nip_address=192.168.1.2")),("SAVE_CONFIG",Ok("OK"))]);assert_eq!(connect(&m,&network(),Some("password".into()),&AtomicBool::new(false)).unwrap(),"Connected; network saved");m.done();}
    #[test]#[ignore="Requires a Linux runtime allowing AF_UNIX datagram sockets"]fn socket_roundtrip(){let dir=tempfile::tempdir().unwrap();let remote=dir.path().join("server");let server=UnixDatagram::bind(&remote).unwrap();let t=thread::spawn(move||{let mut b=[0;64];let(n,peer)=server.recv_from(&mut b).unwrap();assert_eq!(&b[..n],b"PING");server.send_to(b"PONG\n",peer.as_pathname().unwrap()).unwrap();});let c=Control::at(&remote).unwrap();let local=c.local.clone();assert_eq!(fs::metadata(&local).unwrap().permissions().mode()&0o777,0o600);assert_eq!(c.command("PING").unwrap(),"PONG");drop(c);assert!(!local.exists());t.join().unwrap();}
    #[test]fn art_screen_shows_failures_from_each_system_and_hides_successes(){
        use crate::art_history::{History,ResultKind};
        let d=tempfile::tempdir().unwrap();
        let carts=[art_cart(Platform::Sms,"Sonic",None),art_cart(Platform::Gg,"Sonic",None),
            art_cart(Platform::Sms,"Saved",Some(d.path().join("saved.png"))),art_cart(Platform::Sms,"New",None)];
        let mut h=History::default();
        for (cart,result) in carts.iter().zip([ResultKind::Failed,ResultKind::Missing,ResultKind::Found]){h.record(d.path(),cart,result,false,None).unwrap();}
        let mut m=WifiMenu::cart_art(d.path(),&carts);m.input(Action::GbaDown(Btn::A));
        if let Ok(dir)=std::env::var("SLOT_ART_PREVIEW_DIR"){
            fs::create_dir_all(&dir).unwrap();let f=m.render();let mut e=png::Encoder::new(fs::File::create(Path::new(&dir).join("failures.png")).unwrap(),640,480);
            e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();
        }
        let l=m.list.as_ref().unwrap();assert_eq!(l.visible.len(),2);assert_eq!(l.scoped_carts().len(),4);
        let sms=l.scopes.iter().position(|p|*p==Some(Platform::Sms)).unwrap();
        let l=m.list.as_mut().unwrap();l.select(sms);assert_eq!(l.visible.len(),1);assert_eq!(l.scoped_carts().len(),3);
        assert_eq!(l.carts[l.visible[0]].platform,Platform::Sms);
        l.fetched(&Fetched{platform:Platform::Sms,stem:"Sonic".into(),label:Some(d.path().join("Sonic.png"))});l.refresh_visible();assert!(l.visible.is_empty());
        assert!(!m.input(Action::GbaDown(Btn::B)),"B returns to systems");
        assert!(m.input(Action::GbaDown(Btn::B)),"B closes systems");
    }
    #[test]fn render_fixture(){let s=WifiMenu{root:card_root(),transfer:None,transfer_progress:None,rescan:None,worker:None,face:None,dirty:true,on:true,status:"Password is case sensitive".into(),networks:Vec::new(),row:0,keyboard:Some(Keyboard{target:KeyTarget::Wifi(network()),secret:true,password:"Test".into(),key:0,page:0,typed:Some(Instant::now())}),refreshed:Instant::now(),art:None,fetched:Vec::new(),list:None,ra:None,query:None,game_index:None,game_page:0,update:None};
        let mut s=s;
        // SLOT_WIFI_PREVIEW=<dir>: one picture per keyboard page.
        for page in 0..KEYS.len(){
            if let Some(k)=&mut s.keyboard{k.page=page;k.key=12;}
            let f=s.render();assert_eq!(f.rgba.len(),640*480*4);
            if let Ok(dir)=std::env::var("SLOT_WIFI_PREVIEW"){let mut e=png::Encoder::new(fs::File::create(Path::new(&dir).join(format!("keyboard-{page}.png"))).unwrap(),640,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();}
        }
    }
}
