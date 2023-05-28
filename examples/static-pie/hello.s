
.data

msg: .asciz "Hello world!\n"
msg_len = . - msg

.text

.globl _start
_start: 
    mov x0, 1
    adr x1, msg
    ldr x2, =msg_len
    mov x8, 64
    svc #0

    mov x8, 93
    mov x0, 0
    svc #0
        