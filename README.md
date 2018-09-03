# cat
## Introduction
This is my first program in Rust, so for this purpose I've chosen a well-known, simple and open source application: `cat`.  
In my Rust implementation all GNU cat's flags are available and work exactly like the original, so you can consider "my" cat as a drop-in replacement for it, excluding error messages (see _What's bad_ paragraph).  

## What's bad
Currently, this program will stop at the first file error instead of printing a message and skipping it, like the original `cat`. This is not something I want to improve now as I want to move to other projects. Feel free to subit a PR if you wish, otherwise I'll probably fix this behavior in future.  

## Misc
I'd really like to thank the Rust Reddit community, it's been very helpful to me. You can see youself their great job in [this discussion](https://www.reddit.com/r/rust/comments/97vgt1/ive_written_my_first_rust_application_cat_yeah).
