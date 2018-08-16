# cat
## Introduction
This is my first program in Rust, so for this purpose I've chosen a well-known, simple and open source application: `cat`.  
In my Rust implementation all GNU cat's flags are available and work exactly like the original, so you can consider "my" cat as a drop-in replacement for it, excluding error messages (see _What's bad_ paragraph).  

## What's bad
Needless to say, being it my first Rust program, it surely has some code flaws: it could not be optimized enough or be missing some common and useful projects structure. In a word, my code may not be "rusty" enough!  
Here's a list of knows issues:
* `cat -A file` is sensibly slower than GNU cat. I don't know the reason at the moment. It could be some terrible code, or LLVM's fault…
* I really dislike how messages are reported to the user. This is due to a combination of lazyness, inexperience and Rust not being comfortable enough at handing I/O errors. For example, it's just not comfortable to report to the user **what** file could not be open. Take a look at my code and you'll figure out that my error handling code is terrible.  

Of course, everyone is invited to contributing to my code. Speaking about me instead, I'll probably improve this repository in time.