; count.s - Count from 0 to 10 in the accumulator
; Assemble with: ../bin/cl65 -t none -C emu.cfg -o count.bin count.s

.segment "CODE"

reset:
    lda #$00        ; A = 0
loop:
    clc             ; Clear carry
    adc #$01        ; A = A + 1
    cmp #$0A        ; Compare with 10
    bne loop        ; If not equal, loop
    brk             ; Stop

; Interrupt handlers (just loop back to reset for now)
nmi:
irq:
    rti

.segment "VECTORS"
    .word nmi       ; NMI vector ($FFFA)
    .word reset     ; Reset vector ($FFFC)
    .word irq       ; IRQ/BRK vector ($FFFE)
