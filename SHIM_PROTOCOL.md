# Shim Protocol

This protocol is used for the PIO code to communicate with the shim. This is done by an IN followed by a PUSH. Any result will be
stored into the TX FIFO.

Every command is a single u32. The left-most three bits dictate the type of command:

    000: 
    001: 
    010: 
    011: Output
    100: Read data
    101: Write data RAM
    110: Allocate memory
    111: Read input

Output: Outputs to screen. Data is 565 RGB in the last 16 bits. The bits directly after the command type is the 7-bit X coordinate
and the 6-bit Y coordinate.

Read data: Read data that can't be stored in the PIO code. Reads 32-bit words. The bit after the command is whether it's static
data from DOOM.dat or dynamic data in RAM. The remaining 28 bits of the command are used as the address.

Write data RAM: Store data into RAM. Last 16 bits are the data. Address is the remaining 13 bits.

Allocate memory: Allocate some memory. The last 8 bits is the size of the region. Address is returned as a 28 bit address.

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
