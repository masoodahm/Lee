/*
    Copyright (C) 2015 subliun <subliunisdev@gmail.com>
    Copyright © 2015 Zetok Zalbavar <zetok@openmailbox.org>
    All Rights Reserved.

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/


/*
    Binding to toxcore
*/
extern crate rstox;
use rstox::core::*;


/*
    For markov chain
*/
extern crate markov;
use markov::Chain;

/*
    For extra fun with markov chain
*/
extern crate chrono;
use chrono::UTC;

extern crate rand;
use rand::ThreadRng;
use rand::Rng;

/*
    Needed for checking hashes of received messages
*/
use std::hash::{Hash, Hasher, SipHasher};

/*
    Lee's own stuff
*/
// TODO: when other functions will be moved from main.rs, things should be
//       added here
mod bootstrap;
mod for_files;



/*
    For bot functionality
*/
//#[derive(Debug)]   // can't be used, since `rand` doesn't want to cooperate
struct Bot {
    /**
        Tox struct.
    */
    tox: Tox,

    /**
        Bot name.
    */
    // TODO: load from config file
    name: String,

    /**
        Markov chain of strings received from groupchat, friends and
        fed from file.
    */
    markov: Chain<String>,

    /**
        Vector with hashes of received messages.
    */
    hashes: Vec<u64>,

    /**
        Time since last save.
    */
    last_save: i64,

    /**
        Last group from which message of any kind was received.

        This value is being used to decide in which groupchat Lee should
        speak randomly – since out of all groupchats this was the last one
        in which activity was observed, it is most likely that there are
        some people in it able to receive Lee's message.
    */
    last_group: i32,

    /**
        Time since Lee last spoken randomly.
    */
    last_time: i64,

    /**
        Option to allow Lee talk ar $random_interval, it does not affect Lee's
        response when triggered (highlighted).

        Can be altered by users using commands:
         - `.stahp` – will make Lee stop speaking randomly
         - `.talk`  – will make Lee resume speaking randomly

        Defalut value should be `true`.
    */
    speak: bool,

    /**
        `trigger` is used to launch Lee's talk when something will trigger
        it, by mentioning its name. Answer shouldn't be instantaneous, which
        will make Lee more human.

        By default should be `false`, and after countdown was down to 0, it
        should be restored to `false`.
    */
    trigger: bool,

    /**
        Time when trigger happened, as UNIX time in i64.

        Seconds should be added to this value, so that time of Lee's response
        for trigger would be more human-like, rather than instantaneous.
    */
    trigger_time: i64,

    /**
        Cached RNG, apparently it helps with RNG's performance when it's used
        a lot.
    */
    random: ThreadRng,
}


impl Bot {
    /**
        Create new `Bot` struct.

        Takes data for toxcore's instance to load.
    */
    fn new(data: Option<Vec<u8>>) -> Self {
        Bot {
            tox: Tox::new(ToxOptions::new(), data.as_ref()
                                            .map(|x| &**x)).unwrap(),

            name: "Lee".to_string(),
            markov: for_files::make_chain("markov.json"),
            hashes: vec![],
            last_save: UTC::now().timestamp(),
            last_group: 0,
            last_time: UTC::now().timestamp(),
            speak: true,
            trigger: false,
            trigger_time: UTC::now().timestamp(),
            random: rand::thread_rng(),
        }
    }

    /**
        Check whether hash of a string exists.

        If it does, return early `None`.

        If it doesn't, add it to existing ones and return string.
    */
    fn check_hash(&mut self, message: String) -> Option<String> {
        let mut hasher = SipHasher::default();
        message.hash(&mut hasher);
        let hashed = hasher.finish();
        for h in &self.hashes {
            if h == &hashed {
                return None;
            }
        }
        self.hashes.push(hashed);
        Some(message)
    }

    /**
        Add string to markov chain if wasn't already added
    */
    fn add_to_markov(&mut self, message: &str) {
        if let Some(msg) = self.check_hash(message.to_string()) {
            self.markov.feed_str(&msg);
        }
    }


    /**
        Control status message.

        Takes an `Option<String>` as an argument, in a case where it's
        `None`, default status message is being used, otherwise status message
        is being changed to the new one, supplied `String`.
    */
    fn status_message(&mut self, message: Option<String>) {
        match message {
            Some(m) => {
                drop(self.tox.set_status_message(&m));
                println!("{}: Status message set to: \"{}\"",
                         UTC::now(), m);
            },
            None => {
                drop(self.tox.set_status_message("Send me a message 'invite' to get into the groupchat"));
                println!("{}: Status message set to default one",
                         UTC::now());
            },
        }
    }
}



/*
    Defend honour of a bot.
    As extended measure, compares public key of peer.
*/
const FAKE_NAMES: &'static [&'static str] = &["Lee", "Lee\0"];



/*
    Function to deal with incoming friend requests

    Currently accepts all by default
*/
// TODO: make it configurable to accept all / only selected FRs
fn on_friend_request(tox: &mut Tox, fpk: PublicKey, msg: String) {
    drop(tox.add_friend_norequest(&fpk));
    println!("{}: Friend {} with friend message {:?} was added.",
            UTC::now(), fpk, msg);
}


/*
    Function to deal with friend messages.

    Lee is supposed to answer all friend messages, in ~similar way to
    how it's done in groupchats.

    The only **exception** is inviting friends to last groupchat in which
    someone spoke in - in this case Lee should return early.
*/
fn on_friend_message(bot: &mut Bot, fnum: u32, msg: String) {
    let pubkey = match bot.tox.get_friend_public_key(fnum) {
        Some(pkey) => pkey,
        None       => bot.tox.get_public_key(),
    };

    /*
        Invite friend and return early, to not feed markov with invite
        command.

        TODO: make it possible to print to stdout friend's name when inviting
    */
    if &msg == "invite" {
        drop(bot.tox.invite_friend(fnum as i32, bot.last_group));
        println!("{}: Sent invitation to friend {} to groupchat {}",
            UTC::now(), fnum, bot.last_group);
        return;
    }

    println!("{}: Event: FriendMessage:\nFriend {} sent message: {}",
            UTC::now(), pubkey, &msg);

    /*
        feed Lee with message content, but only if peer PK doesn't match
        Lee's own PK

        Feeding Lee with what it threw up may not be a good idea after all..
    */
    if pubkey != bot.tox.get_public_key() {
        bot.add_to_markov(&msg);
    }


    /*
        Send "about" message
    */
    if msg == ".about" || msg == ".help" {
        let message = format!(
"Lee is libre software, licensed under GPLv3+.

Uses Supreme Tox technology.

Made by Zetok\0.
Many thanks to all the people who helped in making it.

For more info, visit: https://gitlab.com/zetok/Lee");
        drop(bot.tox.send_friend_message(fnum, MessageType::Normal, &message));
        println!("{}: Sent \"About\" message to friend {}", UTC::now(), fnum);
    } else {
        let message = bot.markov.generate_str();
        println!("Answer: {}", &message);
        drop(bot.tox.send_friend_message(fnum, MessageType::Normal, &message));
        println!("{}: Sent random message to friend {}", UTC::now(), fnum);
    }
}



/*
    Function to deal with incoming invites to groupchats
*/
fn on_group_invite(tox: &mut Tox, fid: i32, kind: GroupchatType, data: Vec<u8>) {
    /*
        Since rstox currently supports only text groupchats, handle only them,
        and drop other invites.
    */
    match kind {
        GroupchatType::Text => {
            drop(tox.join_groupchat(fid, &data));
            println!("{}: Accepted invite to text groupchat by {}.",
                    UTC::now(), fid);
        },
        GroupchatType::Av => {
            println!("{}: Declined invite to audio groupchat by {}.",
                    UTC::now(), fid);
        },
    }
}


/*
    Function to deal with group messages
*/
fn on_group_message(bot: &mut Bot, gnum: i32, pnum: i32, msg: String) {
    /*
        Get PK of the peer who sent message

        In case where toxcore doesn't feel like providing it, use own PK,
        to avoid triggering false alarm
    */
    let pubkey = match bot.tox.group_peer_pubkey(gnum, pnum) {
        Some(pkey) => pkey,
        None       => bot.tox.get_public_key(),
    };


    // mark this groupchat as last active one
    bot.last_group = gnum;


    /*
        Triggers Lee
    */
    fn trigger_response(msg: &String, bot: &mut Bot) {
        // check whether name is mentioned — convert message to lowercase and
        // then look for lowercase name of bot in message
        if msg.to_lowercase().contains(&bot.name.to_lowercase()) {
            bot.trigger = true;
            /*
                ↓ waiting time for response should be random, for more
                human-like feel, and should be at least 2s long – too
                quick answer isn't too good either.

                Currently waiting time should be between 1 and 5s.
            */
            let random_wait = 1.0 + 4.0 * bot.random.gen::<f64>();
            bot.trigger_time = random_wait as i64 + UTC::now().timestamp();
        }
    }

    match bot.tox.group_peername(gnum, pnum) {
        Some(pname) => {
            /*
                feed Lee with message content, but only if peer PK doesn't match
                Lee's own PK

                Feeding Lee with what it threw up may not be a good idea after
                all..
            */

            if pubkey != bot.tox.get_public_key() {
                bot.add_to_markov(&msg);
            }

            if FAKE_NAMES.contains(&&*pname) {
                if pubkey != bot.tox.get_public_key() {
                    drop(bot.tox.group_message_send(gnum, "↑ an impostor!"));
                }
            }


            if pubkey != bot.tox.get_public_key() {
                trigger_response(&msg, bot);
            }

            println!("{}: Event: GroupMessage({}, {}, {:?}), Name: {:?}, PK: {}",
                    UTC::now(), gnum, pnum, msg, pname, pubkey);
        },

        None => {
            if pubkey != bot.tox.get_public_key() {
                trigger_response(&msg, bot);
            }

            println!("{}: Event: GroupMessage({}, {}, {:?}), Name: •not known•, PK: {}",
                    UTC::now(), gnum, pnum, msg, pubkey);
        },
    }

    /*
        Allow anyone to turn speaking `on / off`, and if switch is changed,
        alter status message accordingly.
    */
    if msg == ".stahp" {
        if bot.speak == true {
            bot.speak = false;
            let new_status = format!("{} | groupchat talk: off",
                                     bot.tox.get_status_message());
            bot.tox.set_status(UserStatus::Away);
            bot.status_message(Some(new_status));
        }
    } else if msg == ".talk" {
        if bot.speak == false {
            bot.speak = true;
            bot.tox.set_status(UserStatus::None);
            bot.status_message(None);
        }
    }

    /*
        Allow anyone to get Lee's ID
    */
    if msg == ".id" && pubkey != bot.tox.get_public_key() {
        let message = format!("My ID: {}", bot.tox.get_address());
        drop(bot.tox.group_message_send(gnum, &message));
    }

    /*
        Send "about" message
    */
    if msg == ".about" || msg == ".help" {
        let message = format!(
"Lee is libre software, licensed under GPLv3+.

Made by Zetok\0.
Many thanks to all the people who helped in making it.

For more info, visit: https://github.com/zetok/Lee");
        drop(bot.tox.group_message_send(gnum, &message));
    }
}


/*
    Function to deal with namechanges in groupchat

    Upon detecting that someone leaves, bot should check how many peers are
    left, and if there is only 1 peer (bot), automatically leave groupchat.

    After leaving groupchat, print info about it.

    In case of other event, print info about it.
*/
fn on_group_namelist_change(tox: &mut Tox, gnum: i32, pnum: i32,
                            change: ChatChange) {
    if let ChatChange::PeerDel = change {
        println!("{}: Event: Groupchat {}, Peer {} left.",
                UTC::now(), gnum, pnum);
        if let Some(peers) = tox.group_number_peers(gnum) {
            if peers == 1 {
                drop(tox.del_groupchat(gnum));
                println!("{}: Left empty group {}.", UTC::now(), gnum);
            }
        }
    } else {
        println!("{}: Event: Groupchat {}, Peer {}: {:?}",
                UTC::now(), gnum, pnum, change);
    }
}


fn main() {
    /*
        Try to load data file, if not possible, print an error and generate
        new Tox instance.
    */
    let data = for_files::load_save("lee.tox").ok();

    /*
        Bot stuff
    */
    let mut bot = Bot::new(data);

    drop(bot.tox.set_name(&bot.name));
    bot.status_message(None);


    /*
        Boostrapping process
        During bootstrapping one should query random bootstrap nodes from a
        supplied list; in case where there is no list, rely back on hardcoded
        bootstrap nodes.
        // TODO: actually make it possible to use supplied list; location of a
        //       list should be determined by value supplied in config file;
        //       in case of absence of config file, working dir should be
        //       tried for presence of file named `bootstrap.txt`, only if it
        //       is missing fall back on hardcoded nodes
    */
    bootstrap::bootstrap_hardcoded(&mut bot.tox);

    println!("\nMy ID: {}", bot.tox.get_address());
    println!("My name: {:?}", bot.tox.get_name());

    loop {
        for ev in bot.tox.iter() {
            match ev {
                FriendRequest(fpk, msg) => {
                    on_friend_request(&mut bot.tox, fpk, msg);
                },

                FriendMessage(fnum, _, msg) => {
                    on_friend_message(&mut bot, fnum, msg);
                },

                GroupInvite(fid, kind, data) => {
                    on_group_invite(&mut bot.tox, fid, kind, data);
                },

                GroupMessage(gnum, pnum, msg) => {
                    on_group_message(&mut bot, gnum, pnum, msg);
                },

                GroupNamelistChange(gnum, pnum, change) => {
                    on_group_namelist_change(&mut bot.tox, gnum, pnum, change);
                },

                ev => { println!("{}: Event: {:?}", UTC::now(), ev); },
            }
        }


        /*
            Let Lee speak when triggered, provided that it will wait required
            amount of time.
        */
        if bot.trigger {
            let cur_time = UTC::now().timestamp();
            if cur_time >= bot.trigger_time {
                let message = bot.markov.generate_str();
                drop(bot.tox.group_message_send(bot.last_group, &message));
                bot.trigger = false;
            }
        }


        /*
            Let Lee speak every $time_interval, provided that there is given
            permission for it
        */
        if bot.speak {
            let cur_time = UTC::now().timestamp();
            if  (bot.last_time + 10) < cur_time {
                /* Should have only small chance to speak */
                if 0.01 > bot.random.gen::<f64>() {
                    let message = bot.markov.generate_str();
                    drop(bot.tox.group_message_send(bot.last_group, &message));
                }

                bot.last_time = cur_time;
            }
        }


        /*
            Write save data every 64s.

            After a write, be it successful or not, set clock again to tick,
            for the next time when it'll need to be saved.
            TODO: save data every $relevant_event, rather than on timer.
        */
        let cur_time = UTC::now().timestamp();
        if bot.last_save + 64 < cur_time {
            match for_files::write_save("lee.tox", bot.tox.save()) {
                Ok(_) => println!("{}: File saved.", UTC::now()),
                Err(e) => println!("\n{}: Failed to save file: {}",
                                UTC::now(), e),
            }
            drop(bot.markov.save_utf8("markov.json"));
            println!("{}: Saved `markov.json`", UTC::now());
            bot.last_save = cur_time;
        }


        bot.tox.wait();
    }
}
