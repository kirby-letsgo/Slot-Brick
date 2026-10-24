//! Boot and wake timing for Slot/Logs/boot-times.log. Each step of a start (the startup hook,
//! boot.sh, launch.sh, then Slot) or of a wake (sleep.sh, then Slot) appends `<uptime> <step>`
//! to a trace in /tmp. Once the shelf is usable, or the screen is back after a wake, Slot turns
//! the trace into one block of the log.
use std::collections::HashSet;

pub const BOOT_TRACE: &str = "/tmp/slot-boot-trace";
pub const WAKE_TRACE: &str = "/tmp/slot-wake-trace";
const KERNEL_RETURNED: &str = "sleep.sh: kernel returned";

pub type Step = (f64, String);

/// The trace's `<uptime> <step>` lines in uptime order; lines that don't parse are dropped.
pub fn parse_steps(trace: &str) -> Vec<Step> {
    let mut steps: Vec<Step> = trace
        .lines()
        .filter_map(|line| {
            let (up, step) = line.trim().split_once(' ')?;
            let up: f64 = up.parse().ok().filter(|u: &f64| u.is_finite())?;
            let step = step.trim();
            (!step.is_empty()).then(|| (up, step.to_string()))
        })
        .collect();
    steps.sort_by(|a, b| a.0.total_cmp(&b.0));
    steps
}

pub fn boot_block(header: &str, steps: &[Step]) -> String {
    let mut out = format!("{header}\n{:>9} {:>7}   step\n", "uptime", "+step");
    let mut last = 0.0;
    for (up, step) in steps {
        out += &format!("{up:>9.2} {:>7.2}   {step}\n", up - last);
        last = *up;
    }
    out
}

/// Milliseconds from the kernel returning (else from the first step), each with the time
/// since the step before.
pub fn wake_block(header: &str, steps: &[Step]) -> String {
    let start = steps
        .iter()
        .find(|(_, s)| s == KERNEL_RETURNED)
        .or(steps.first())
        .map_or(0.0, |(up, _)| *up);
    let mut out = format!("{header}\n");
    let mut last: Option<i64> = None;
    for (up, step) in steps {
        let ms = ((up - start) * 1000.0).round() as i64;
        let delta = last.map_or(String::new(), |l| format!("+{}", ms - l));
        out += &format!("{ms:>6} ms  {delta:<6}  {step}\n");
        last = Some(ms);
    }
    out
}

/// The log with `block` added, keeping the last `keep` blocks (blank-line separated).
pub fn keep_last(log: &str, block: &str, keep: usize) -> String {
    let mut blocks: Vec<&str> = log
        .split("\n\n")
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .collect();
    blocks.push(block.trim());
    let from = blocks.len().saturating_sub(keep);
    blocks[from..].join("\n\n") + "\n"
}

/// `/proc/<pid>/stat`: the command name, parent PID and start time in clock ticks after boot.
/// The name sits in brackets and may hold spaces and brackets itself, so the fields after it
/// are found from the last `)`.
pub fn parse_stat(stat: &str) -> Option<(String, u32, u64)> {
    let open = stat.find('(')?;
    let close = stat.rfind(')')?;
    let name = stat.get(open + 1..close)?.to_string();
    let rest: Vec<&str> = stat.get(close + 1..)?.split_whitespace().collect();
    // rest[0] is field 3 (state); the parent PID is field 4, the start time field 22.
    Some((
        name,
        rest.get(1)?.parse().ok()?,
        rest.get(19)?.parse().ok()?,
    ))
}

/// The firmware's processes still running that started before `before` (the hook), earliest
/// first and once per name. Kernel threads (kthreadd, PID 2, and its children) are left out.
pub fn firmware_steps(procs: &[(u32, String)], ticks: f64, before: f64) -> Vec<Step> {
    let mut found: Vec<Step> = procs
        .iter()
        .filter_map(|(pid, stat)| {
            let (name, ppid, start) = parse_stat(stat)?;
            (*pid != 2 && ppid != 2).then(|| (start as f64 / ticks, name))
        })
        .filter(|(up, _)| *up < before)
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut seen = HashSet::new();
    found.retain(|(_, name)| seen.insert(name.clone()));
    found
        .into_iter()
        .map(|(up, name)| (up, format!("firmware: {name}")))
        .collect()
}

fn dmesg_time(line: &str) -> Option<f64> {
    let rest = &line[line.find('[')? + 1..];
    rest.split_once(']')?.0.trim().parse().ok()
}

/// When init started and when the SD card appeared, if those lines are still in the kernel log.
pub fn kernel_steps(dmesg: &str) -> Vec<Step> {
    [
        ("Freeing unused kernel memory", "kernel: init started"),
        ("mmc1: new ", "kernel: SD card found"),
    ]
    .iter()
    .filter_map(|(needle, step)| {
        let up = dmesg
            .lines()
            .filter(|l| l.contains(needle))
            .find_map(dmesg_time)?;
        Some((up, step.to_string()))
    })
    .collect()
}

/// The kernel's own device resume time for the last wake, if this kernel logs it.
pub fn kernel_resume_ms(dmesg: &str) -> Option<f64> {
    dmesg.lines().rev().find_map(|l| {
        let after = l.split_once("PM: resume of devices complete after ")?.1;
        after.split_whitespace().next()?.parse().ok()
    })
}

#[cfg(feature = "brick")]
pub use device::{save, stamp, Trace};

#[cfg(feature = "brick")]
mod device {
    use super::*;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    const LOG: &str = "Logs/boot-times.log";
    const KEEP: usize = 20;
    const HOOK_STARTED: &str = "hook: started";

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Trace {
        Boot,
        Wake,
    }

    impl Trace {
        fn path(self) -> PathBuf {
            let (var, default) = match self {
                Trace::Boot => ("SLOT_BOOT_TRACE", BOOT_TRACE),
                Trace::Wake => ("SLOT_WAKE_TRACE", WAKE_TRACE),
            };
            std::env::var_os(var)
                .map(PathBuf::from)
                .unwrap_or_else(|| slot_power::hw(default))
        }
    }

    /// Appends one step. Best-effort: timing must never get in the way of starting or waking.
    pub fn stamp(trace: Trace, step: &str) {
        let raw = std::fs::read_to_string(slot_power::hw("/proc/uptime")).unwrap_or_default();
        let Some(up) = slot_power::uptime_seconds(&raw) else {
            return;
        };
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(trace.path())
        {
            let _ = writeln!(f, "{up} {step}");
        }
    }

    /// Turns the trace into a block of Logs/boot-times.log on a background thread, then clears it.
    pub fn save(root: &Path, trace: Trace) {
        let root = root.to_path_buf();
        let spawned = std::thread::Builder::new()
            .name("slot-boot-times".into())
            .spawn(move || write_block(&root, trace));
        if let Err(e) = spawned {
            eprintln!("slot: boot times: {e}");
        }
    }

    fn write_block(root: &Path, trace: Trace) {
        let path = trace.path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        let _ = std::fs::remove_file(&path);
        let mut steps = parse_steps(&text);
        if steps.is_empty() {
            return;
        }
        let block = match trace {
            Trace::Boot => {
                let hook = steps
                    .iter()
                    .find(|(_, s)| s == HOOK_STARTED)
                    .map(|(up, _)| *up);
                if let Some(hook) = hook {
                    steps.extend(kernel_steps(&dmesg()));
                    steps.extend(firmware_steps(&processes(), clock_ticks(), hook));
                    steps.sort_by(|a, b| a.0.total_cmp(&b.0));
                }
                let header = format!(
                    "boot {} UTC · Slot {} ({})",
                    slot_store::stamp_now(),
                    slot_ui::PORT_VERSION.to_lowercase(),
                    crate::build_info::Build::current().serial()
                );
                boot_block(&header, &steps)
            }
            Trace::Wake => {
                let resume = kernel_resume_ms(&dmesg())
                    .map_or("kernel resume not logged".to_string(), |ms| {
                        format!("kernel resume {ms:.0} ms")
                    });
                let header = format!("wake {} UTC · {resume}", slot_store::stamp_now());
                wake_block(&header, &steps)
            }
        };
        let log = root.join(LOG);
        let old = std::fs::read_to_string(&log).unwrap_or_default();
        let tmp = log.with_extension("log.new");
        if std::fs::write(&tmp, keep_last(&old, &block, KEEP))
            .and_then(|()| std::fs::rename(&tmp, &log))
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
        }
    }

    fn processes() -> Vec<(u32, String)> {
        let Ok(dir) = std::fs::read_dir(slot_power::hw("/proc")) else {
            return Vec::new();
        };
        dir.flatten()
            .filter_map(|e| {
                let pid = e.file_name().to_str()?.parse().ok()?;
                Some((pid, std::fs::read_to_string(e.path().join("stat")).ok()?))
            })
            .collect()
    }

    fn clock_ticks() -> f64 {
        match unsafe { libc::sysconf(libc::_SC_CLK_TCK) } {
            n if n > 0 => n as f64,
            _ => 100.0,
        }
    }

    #[cfg(not(feature = "sim"))]
    fn dmesg() -> String {
        std::process::Command::new("dmesg")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    }

    /// The simulator never reads the kernel log of the machine it runs on.
    #[cfg(feature = "sim")]
    fn dmesg() -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps(list: &[(f64, &str)]) -> Vec<Step> {
        list.iter().map(|(up, s)| (*up, s.to_string())).collect()
    }

    #[test]
    fn steps_are_read_in_uptime_order_and_junk_is_dropped() {
        let trace = "7.18 hook: card mounted\n6.68 hook: started\nnot a line\nnan x\n8.00\n";
        assert_eq!(
            parse_steps(trace),
            steps(&[(6.68, "hook: started"), (7.18, "hook: card mounted")])
        );
    }

    #[test]
    fn boot_block_shows_uptime_and_time_since_the_step_before() {
        let s = steps(&[
            (2.41, "kernel: init started"),
            (6.68, "hook: started"),
            (7.18, "hook: card mounted"),
        ]);
        assert_eq!(
            boot_block("boot X", &s),
            "boot X\n   uptime   +step   step\n     2.41    2.41   kernel: init started\n     6.68    4.27   hook: started\n     7.18    0.50   hook: card mounted\n"
        );
    }

    #[test]
    fn wake_block_counts_milliseconds_from_the_kernel_returning() {
        let s = steps(&[
            (100.0, "sleep.sh: kernel returned"),
            (100.32, "sleep.sh: handed back"),
            (100.78, "slot: first frame after wake"),
        ]);
        assert_eq!(
            wake_block("wake X", &s),
            "wake X\n     0 ms          sleep.sh: kernel returned\n   320 ms  +320    sleep.sh: handed back\n   780 ms  +460    slot: first frame after wake\n"
        );
    }

    #[test]
    fn wake_block_without_the_kernel_line_counts_from_the_first_step() {
        let s = steps(&[(50.0, "slot: helper returned"), (50.1, "slot: woke")]);
        assert_eq!(
            wake_block("wake X", &s),
            "wake X\n     0 ms          slot: helper returned\n   100 ms  +100    slot: woke\n"
        );
    }

    #[test]
    fn the_log_keeps_the_last_blocks() {
        let old = (1..=20)
            .map(|n| format!("boot {n}\n  line"))
            .collect::<Vec<_>>()
            .join("\n\n")
            + "\n";
        let new = keep_last(&old, "wake 21\n  line\n", 20);
        assert!(new.starts_with("boot 2\n  line\n\nboot 3\n"), "{new}");
        assert!(new.ends_with("\n\nwake 21\n  line\n"), "{new}");
        assert_eq!(new.matches("\n\n").count(), 19);
        assert_eq!(keep_last("", "boot 1\n  a\n", 20), "boot 1\n  a\n");
    }

    #[test]
    fn stat_names_may_hold_spaces_and_brackets() {
        let stat =
            "321 (my (odd) name) S 1 321 321 0 -1 4194560 10 0 0 0 1 2 0 0 20 0 1 0 412 1024 50";
        assert_eq!(
            parse_stat(stat),
            Some(("my (odd) name".to_string(), 1, 412))
        );
        assert_eq!(parse_stat("1 (init) S"), None);
    }

    #[test]
    fn firmware_steps_are_user_processes_started_before_the_hook() {
        let stat = |name: &str, ppid: u32, start: u64| {
            format!("9 ({name}) S {ppid} 9 9 0 -1 0 0 0 0 0 0 0 0 0 20 0 1 0 {start} 0 0")
        };
        let procs = vec![
            (1, stat("procd", 0, 210)),
            (2, stat("kthreadd", 0, 0)),
            (40, stat("kworker/0:1", 2, 5)),
            (300, stat("adbd", 1, 412)),
            (310, stat("sh", 1, 500)),
            (311, stat("sh", 1, 520)),
            (400, stat("trimui_inputd", 1, 900)),
        ];
        assert_eq!(
            firmware_steps(&procs, 100.0, 6.68),
            steps(&[
                (2.1, "firmware: procd"),
                (4.12, "firmware: adbd"),
                (5.0, "firmware: sh")
            ])
        );
    }

    #[test]
    fn kernel_steps_come_from_dmesg_when_present() {
        let dmesg = "[    0.000000] Booting Linux\n[    2.412345] Freeing unused kernel memory: 1024K\n<6>[    3.100000] mmc1: new high speed SDXC card at address aaaa\n";
        assert_eq!(
            kernel_steps(dmesg),
            steps(&[
                (2.412345, "kernel: init started"),
                (3.1, "kernel: SD card found")
            ])
        );
        assert_eq!(kernel_steps("[ 1.0] nothing here\n"), Vec::<Step>::new());
    }

    #[test]
    fn kernel_resume_is_the_last_one_logged() {
        let dmesg = "[ 10.0] PM: resume of devices complete after 300.100 msecs\n[ 90.0] PM: resume of devices complete after 412.600 msecs\n";
        assert_eq!(kernel_resume_ms(dmesg), Some(412.6));
        assert_eq!(kernel_resume_ms("[ 1.0] PM: suspend entry\n"), None);
    }
}
