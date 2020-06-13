use std::process::Command;
mod user_notification_chatbot;

fn sleep_secs(s: u64) {
    std::thread::sleep(std::time::Duration::from_secs(s));
}

fn is_freeswitch_active() -> bool {
    let output = Command::new("systemctl")
        .args(&["show", "freeswitch.service", "--no-page"])
        .output()
        .expect("could not run systemctl");
    let output = String::from_utf8(output.stdout).expect("systemctl output is not valid utf-8");
    let is_running = output.contains("SubState=running");
    is_running
}

fn is_freeswitch_working(url: &str) -> bool {
    let mut tries_left = 3;

    while tries_left > 0 {
        let res = reqwest::blocking::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send();

        let mut is_err = false;
        if res.is_err() {
            is_err = true;
        } else if let Ok(res) = res {
            if res.status() != 400 {
                is_err = true;
            }
        }

        if is_err == false {
            return true;
        }

        tries_left -= 1;
        println!("Error polling freeswitch, trying again in 2 seconds…");
        sleep_secs(2);
    }
    return false;
}

fn restart_freeswitch() {
    loop {
        println!("restarting services…");
        let status = Command::new("systemctl")
            .args(&["restart", "freeswitch.service"])
            .status(); // "bbb-fsesl-akka.service"
        let mut restarted = false;
        if status.is_ok() {
            if let Ok(ref status) = status {
                if status.success() {
                    restarted = true;
                }
            }
        }

        if restarted {
            println!("restart complete, waiting 10 minutes");
            sleep_secs(10 * 60);
            return;
        } else {
            println!("error restarting services:\n {:?}", status);
            println!("trying again in 30s");
            sleep_secs(30);
        }
    }
}

fn main() {
    let hostname = Command::new("hostname")
        .arg("-f")
        .output()
        .expect("could not get FQDN");
    let hostname = String::from_utf8(hostname.stdout).expect("hostname is not valid utf-8");
    let hostname = hostname.trim();
    let url = format!("https://{}/ws", hostname);

    let chatbot_notifier = user_notification_chatbot::spawn_chatbot();

    println!("waiting for 10 minutes before beginning checks");
    sleep_secs(10 * 60);
    println!("now starting to periodically poll freeswitch");

    loop {
        // wait until freeswitch is runnning
        while !is_freeswitch_active() {
            sleep_secs(30);
        }

        // check for freeswitch status
        let is_err = !is_freeswitch_working(&url);

        if is_err {
            println!("freeswitch is in invalid state");
            let res = chatbot_notifier.send(());
            if res.is_err() {
                eprint!("error notifying users: {:?}", res.unwrap_err());
            }
            restart_freeswitch();
        } else {
            // everything fine, poll again in 30s
            sleep_secs(30);
        }
    }
}
