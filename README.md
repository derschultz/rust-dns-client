# rust-dns-client
a simple dns client in rust, made while learning rust.

This is the first cut at writing a simple dns client; it is not (and probably will never be) finished.
    This codebase suffers from the fact that I did not plan much of it out before writing it. 
    A rewrite with actual, fleshed-out packages, unit tests, etc. is something I'd like to do next.
        Adding in things like tcp/dot/doh/doq? would be nice, too.

the intent of this code was to write as much from scratch as possible, so as to maximize learning.
much of the functionality here is already implemented (and probably in a more idiomatic and efficient way) in various rust crates.

lessons learned:
- there's a big #[derive ...] boilerplate chunk that is necessary for many/all of your user-defined structs/enums
- while it might be more educational to have done it (somewhat) manually, maybe leave the server addr parsing to the socket functions in the future - a lot of time was spent messing around with the code for parsing the server address when it could have been left as a String for the socket functions to use.
- create structs, impls, etc. first. it's alright to have an overarching idea of what the program should do, but having a base of simple pieces to build on is better than trying to fill them in later, from my experience in writing this code.
- add AXFR/IXFR to the list of "known unknowns" in my dns knowledge

