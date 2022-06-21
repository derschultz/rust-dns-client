# rust-dns-client
a simple dns client in rust, made while learning rust.

the intent of this code is to write as much from scratch as possible, so as to maximize learning.
much of the functionality here is already contained (and probably better-written) in various rust crates.

lessons learned:
-there's a big #[derive ...] boilerplate chunk that basically needs to go on each of your structs/enums
-while it might be more educational to have done it (somewhat) manually, maybe leave the server addr parsing to the socket functions in the future - a lot of time was spent messing around with the code for parsing the server address when it could have been left as a String for the socket functions to use.
