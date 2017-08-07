extern crate inotify;
extern crate mio;

use std::env;
use inotify::*;

fn main() {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    let current_dir = env::current_dir().expect("Failed to determine current directory");
    inotify.add_watch(current_dir, watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE)
        .expect("Failed to add inotify watch");

    const INOTIFY: mio::Token = mio::Token(0);
    let poll = mio::Poll::new().unwrap();
    poll.register(&inotify, INOTIFY, mio::Ready::readable(), mio::PollOpt::edge()).unwrap();

    println!("Watching current directory for activity...");

    let mut mio_events = mio::Events::with_capacity(1024);
    let mut inotify_buf = [0u8; 4096];
    loop {
        poll.poll(&mut mio_events, None).unwrap();
        for mio_event in mio_events.iter() {
            match mio_event.token() {
                INOTIFY => {
                    let inotify_events = inotify
                        .read_events_blocking(&mut inotify_buf)
                        .expect("Failed to read inotify events");

                    for inotify_event in inotify_events {
                        if inotify_event.mask.contains(event_mask::CREATE) {
                            if inotify_event.mask.contains(event_mask::ISDIR) {
                                println!("Directory created: {:?}", inotify_event.name);
                            } else {
                                println!("File created: {:?}", inotify_event.name);
                            }
                        } else if inotify_event.mask.contains(event_mask::DELETE) {
                            if inotify_event.mask.contains(event_mask::ISDIR) {
                                println!("Directory deleted: {:?}", inotify_event.name);
                            } else {
                                println!("File deleted: {:?}", inotify_event.name);
                            }
                        } else if inotify_event.mask.contains(event_mask::MODIFY) {
                            if inotify_event.mask.contains(event_mask::ISDIR) {
                                println!("Directory modified: {:?}", inotify_event.name);
                            } else {
                                println!("File modified: {:?}", inotify_event.name);
                            }
                        }
                    }
                }
                mio::Token(_) => unreachable!(),
            }
        }
    }
}
