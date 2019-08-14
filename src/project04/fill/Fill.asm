// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.
@32
D = A
@row_word
M = D
@16
D = A
@col_word
M = D

(MAINLOOP)
    @i
    M = 0
    @SCREEN
    D = A
    @ptr
    M = D
    (OUTER)
        @j
        M = 0
        (INNER)
            @KBD
            D = M
            @FILLBLACK
            D;JEQ
            @color
            M = 0
            @FILL
            0;JMP
            (FILLBLACK)
                @color
                M = -1
            (FILL)
            @row_word
            D = M
            @j
            D = D - M
            @INNEREND
            D;JEQ
            @color
            D = M
            @ptr
            A = M
            M = D
            @ptr
            M = M + 1
            @j
            M = M + 1
            @INNER
            0;JMP
        (INNEREND)
        @col_word
        D = M
        @i
        D = D - M
        @MAINLOOP
        D;JEQ
        @i
        M = M + 1;
        @OUTER
        0;JMP
