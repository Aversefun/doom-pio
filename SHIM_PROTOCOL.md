# Shim Protocol

This protocol is used for the PIO code to communicate with the shim. This is done by an IN followed by a PUSH. Any result will be
stored into the TX FIFO.

Every command is a single u32. The left-most three bits dictate the type of command:

000: Bank jump
001: Bank call
010: Bank return
011: Output
100: Read data
101: Write data RAM
110: Read data RAM
111: Read input

Bank jump: Last sixteen bits dictate the bank to jump to.

Bank call: Last sixteen bits dictate the bank to call. Current state is saved and restored except for scratch registers, FIFOs,
and ISR/OSR upon return.

Bank return: Restores state from before bank call.

Output: Outputs to screen. Data is 565 RGB in the last 16 bits. The bits directly after the command type is the 7-bit X coordinate
and the 6-bit Y coordinate.

Read data: Read data that can't be stored in the PIO code. Reads 32-bit words. The remaining 29 bits of the command are used as
the address into src/DOOM.dat.

Write data RAM: Store data into RAM. Last 16 bits are the data. Address is returned as a 29-bit word.

Read data RAM: Read data from RAM. Remainder of the bits are the address. Data is returned in the low 16 bits.

Read input: Receive input from the user. Returned in a bit flag:

    Bit 0 (LSB): Turn left
    Bit 1: Turn right
    Bit 2: Move forward
    Bit 3: Move backward
    Bit 4: Strafe left
    Bit 5: Strafe right
    Bit 6: Fire weapon
    Bit 7: Use/open
    Bit 8: Run
