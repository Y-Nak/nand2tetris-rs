# nand2tetris-rs
Solutions of [nand2tetris](https://www.nand2tetris.org/) written by Rust.  

## Description

### Solutions
* `src/project01` : Solutions of project01
* `src/project02` : Solutions of project02
* `src/project03` : Solutions of project03
* `src/project04` : Solutions of project04
* `src/project05` : Solutions of project05
* `src/code_gen` : Sollutions of project06.
* `src/asm_gen`  : Sollutions of project07 and project08.
* `src/project09` : Solutions of project09
* `src/jack`     : Solutions of project10 and project11
* `src/project12`: Solutions of project12

### Run Code generator(project06)
Generate .hack file from .asm file.  
```cargo run --bin code_gen -- -o OUTPUT INPUT```

### Run Asm generator (project07 and project08)
Generate .asm file from .vm file.  
```cargo run --bin asm_gen -- -o  OUTPUT INPUT```  
If initialization code can be omitted, please add `--no-init` option.  
```cargo run --bin asm_gen -- -o  OUTPUT --no-init INPUT```


### Run Parser (project09)
Generate XML that represents AST.  
``` cargo run  --bin parser  -- INPUT ```

### Run Jack compiler (project09 and project10)
Generate .vm file from .jack code.  
``` Cargo run  --bin jackc  -- INPUT ```