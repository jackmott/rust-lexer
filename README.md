# A port of Rob Pike's Go lexer to Rust

Rob Pike has an excellent video showing how to write a nice lexical scanner in golang: https://www.youtube.com/watch?v=HxaD_trXwRE
This is a port of that approach to Rust. This is just a sample binary you can play with, not meant as a general purpose library. 
The lexer sends tokens on a channel so that a parser could consume them from a separate thread.  No string allocations are made, 
tokens refer to slices of the original input string.


