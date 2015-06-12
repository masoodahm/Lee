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



extern crate rstox;

use rstox::core::*;

// to bootstrap
mod bootstrap;

/*
    Bot name.
*/
// TODO: load it from config file, if not available, then use default one
//         * perhaps it could be made of some random chars generated
//           at runtime?
static BOT_NAME: &'static str = "Layer ";


fn main() {
    /*
        Defend my honour. Needed to compare whether someone is not trying to
        use my nick.
        Also defend bot.
    */
    let name1 = "Zetok".to_string();
    let name2 = "zetok".to_string();
    let name3 = "Layer".to_string();

    /*
        Initialize tox instance (or whatever it's called)
        // TODO: fix comment ↑, so that proper wording would be used
    */
    let mut tox = Tox::new(ToxOptions::new(), None).unwrap();


    /*
        Set bot name.
    */
    tox.set_name(BOT_NAME).unwrap();


    /*
        Boostrapping process
        During bootstrapping one should query random bootstrap nodes from a
        supplied list; in case where there is no list, rely back on hardcoded
        bootstrap nodes.
        // TODO: actually make it possible to use supplied list
    */
    bootstrap::bootstrap_hardcoded(&mut tox);


    println!("{}", tox.get_address());

    loop {
        for ev in tox.iter() {
            match ev {
                FriendRequest(cid, _) => {
                    tox.add_friend_norequest(&cid).unwrap();
                },
                GroupInvite(fid, kind, data) => {
                    match kind {
                        GroupchatType::Text => {
                            tox.join_groupchat(fid, &data).unwrap();
                        },
                        _ => {},
                    }
                },
                GroupMessage(gnum, pnum, msg) => {
                    match tox.group_peername(gnum, pnum) {
                        Some(pname) => {
                            if pname == name1 || pname == name2 || pname == name3 {
                                let _ = tox.group_message_send(gnum, "↑ an impostor!");
                                println!("Tox event: GroupMessage({}, {}, {:?}), Name: {:?}", gnum, pnum, msg, pname);
                            } else {
                                println!("Tox event: GroupMessage({}, {}, {:?}), Name: {:?}", gnum, pnum, msg, pname);
                            }
                        },
                        None => {
                            println!("Tox event: GroupMessage({}, {}, {:?}), Name: •not known•",
                                gnum, pnum, msg);
                        },
                    }
                },
                GroupNamelistChange(gnum, pnum, change) => {
                    match change {
                        peer  => {
                            if peer == ChatChange::PeerAdd {
                                let msg = format!("Peer {} joined.", pnum);
                                let _ = tox.group_message_send(gnum, &msg);
                            } else if peer == ChatChange::PeerDel {
                                let msg = format!("Peer {} left.", pnum);
                                let _ = tox.group_message_send(gnum, &msg);
                            } else if peer == ChatChange::PeerName {
                                match tox.group_peername(gnum, pnum) {
                                    Some(pname) => {
                                        let msg = format!("Peer {} is now known as {}", pnum, pname);
                                        let _ = tox.group_message_send(gnum, &msg);
                                    },
                                    None => {
                                        let msg = format!("Peer {} has unknown name!", pnum);
                                        let _ = tox.group_message_send(gnum, &msg);
                                    },
                                }
                            }
                        },
                    }

                },
                            
                /*GroupTitle(gnum, _, _) => {
                    let _ = tox.group_set_title(gnum, "#tox-real-ontopic ");
                },*/
                ev => { println!("Tox event: {:?}", ev); },
            }
        }
        let _ = tox.group_set_title(0, "░▒▓█ #tox-real-ontopic | No. █▓▒░");


        tox.wait();
    }
}
