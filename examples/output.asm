            Blez 	zero, zero, main
            Halt			
main        Add 	zero, zero, zero		
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		ra, 0[sp] 	(0x0000)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		fp, 0[sp] 	(0x0000)
            Add 	fp, zero, sp		
            Addi 	t0, zero, 10 	(0x000a)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Addi 	t0, zero, 0 	(0x0000)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Lw 		zero, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Lw 		t0, -4[fp] 	(0xfffc)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Lw 		t0, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Add 	sp, zero, fp		
            Lw 		fp, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Lw 		ra, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Add 	v0, zero, t0		
            Jr 	ra		
add         Add 	zero, zero, zero		
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		ra, 0[sp] 	(0x0000)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		fp, 0[sp] 	(0x0000)
            Add 	fp, zero, sp		
            Lw 		t0, 12[fp] 	(0x000c)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Lw 		t0, 8[fp] 	(0x0008)
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Lw 		t1, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Lw 		t0, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Add 	t0, t0, t1		
            Addiu 	sp, sp, -4 	(0xfffc)
            Sw 		t0, 0[sp] 	(0x0000)
            Lw 		t0, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Add 	sp, zero, fp		
            Lw 		fp, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Lw 		ra, 0[sp] 	(0x0000)
            Addiu 	sp, sp, 4 	(0x0004)
            Add 	v0, zero, t0		
            Jr 	ra		
