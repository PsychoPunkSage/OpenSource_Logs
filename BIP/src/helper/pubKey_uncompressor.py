def compressed_pubkey_to_uncompressed(compressed):
    # Split compressed key into prefix and x-coordinate
    prefix = compressed[:2]
    x = int(compressed[2:], 16)

    # Secp256k1 curve parameters
    p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f

    # Work out y values using the curve equation y^2 = x^3 + 7
    y_sq = (x**3 + 7) % p  # everything is modulo p

    # Secp256k1 is chosen in a special way so that the square root of y is y^((p+1)/4)
    y = pow(y_sq, (p+1)//4, p)  # use modular exponentiation

    # * 02 prefix = y is even
    # * 03 prefix = y is odd
    if prefix == "02" and y % 2 != 0:  # if prefix is 02 and y isn't even, use other y value
        y = (p - y) % p
    if prefix == "03" and y % 2 == 0:  # if prefix is 03 and y is even, use other y value
        y = (p - y) % p

    # # Convert x and y to hex strings and ensure they are 32 bytes (64 characters)
    # x_hex = format(x, '064x')
    # y_hex = format(y, '064x')

    # # Construct the uncompressed public key
    # uncompressed = "04" + x_hex + y_hex

    return (x, y)

"""
Example::> 
Pubkey: 03ab996ad23c7930cee68f950e739fa067aa70a0e63786572b864900985879c4c4
            |_> x: 77616561961719797560395316518092500847148122687187451311177913683967720670404
            |_> y: 69589226102335479499252171152551221549584577390107066575000126767261437088529
"""