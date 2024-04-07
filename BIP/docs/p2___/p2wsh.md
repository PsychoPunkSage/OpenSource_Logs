## p2wsh txn

>> The idea here was that the payer—the donor, in this case—shouldn’t have to pay a higher fee for a big, complex pubkey script. Instead, the recipient wanting to use this fancy scheme will pay for the complexity.



They use this witness script hash to create a p2wsh address in the same way you created your p2wpkh address. They encode `00 983b977f86b9bce124692e68904935f5e562c88226befb8575b4a51e29db9062` using Bech32 and get the p2wsh address: `bc1qnqaewluxhx7wzfrf9e5fqjf47hjk9jyzy6l0hpt4kjj3u2wmjp3qr3lft8`

### Verification

> 1. A full node that wants to verify this transaction needs to determine the type of output being spent. It looks at the output, finds the pattern **\<version byte> <2 to 40 bytes data>**, and concludes that this is a segwit output. 
> 2. The next thing to check is the value of the version byte.

![segwit hash](../image/segwit_hash.png)

> 3. Special rules apply when spending a p2wsh output. First, the data items in the spending input’s witness field are pushed onto the program stack. Then, the top item on the stack, the witness script, is verified against the witness program in the output