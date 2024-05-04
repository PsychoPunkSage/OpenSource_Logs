'use strict';
Object.defineProperty(exports, '__esModule', { value: true });
exports.Block = void 0;
const bufferutils_1 = require('./bufferutils');
const bcrypto = require('./crypto');
const merkle_1 = require('./merkle');
const transaction_1 = require('./transaction');
const types = require('./types');
const { typeforce } = types;
const errorMerkleNoTxes = new TypeError(
  'Cannot compute merkle root for zero transactions',
);
const errorWitnessNotSegwit = new TypeError(
  'Cannot compute witness commit for non-segwit block',
);
class Block {
  // Initialize block properties
  constructor() {
    this.version = 1;
    this.prevHash = undefined;
    this.merkleRoot = undefined;
    this.timestamp = 0;
    this.witnessCommit = undefined;
    this.bits = 0;
    this.nonce = 0;
    this.transactions = undefined;
  }

  // Method to create a Block instance from a buffer
  // <<<REQUIRED>>>
  static fromBuffer(buffer) {
    if (buffer.length < 80) throw new Error('Buffer too small (< 80 bytes)');
    const bufferReader = new bufferutils_1.BufferReader(buffer);
    const block = new Block();
    // Parse block header fields
    block.version = bufferReader.readInt32();
    block.prevHash = bufferReader.readSlice(32);
    block.merkleRoot = bufferReader.readSlice(32);
    block.timestamp = bufferReader.readUInt32();
    block.bits = bufferReader.readUInt32();
    block.nonce = bufferReader.readUInt32();
    // If buffer length is exactly 80 bytes, return the block
    if (buffer.length === 80) return block;
    const readTransaction = () => {
      // Parse transaction from buffer
      const tx = transaction_1.Transaction.fromBuffer(
        bufferReader.buffer.slice(bufferReader.offset),
        true,
      );
      // Increment buffer offset
      bufferReader.offset += tx.byteLength();
      return tx;
    };

    // Read the number of transactions in the block
    const nTransactions = bufferReader.readVarInt();
    block.transactions = []; // Initialize transactions array

    // Iterate over transactions and parse them
    for (let i = 0; i < nTransactions; ++i) {
      const tx = readTransaction();
      block.transactions.push(tx);
    }
    const witnessCommit = block.getWitnessCommit();
    // This Block contains a witness commit
    if (witnessCommit) block.witnessCommit = witnessCommit;
    return block;
  }

  // Method to create a Block instance from a hexadecimal string
  static fromHex(hex) {
    return Block.fromBuffer(Buffer.from(hex, 'hex'));
  }

  // <<DONE>> This is given. So no need to make this
  static calculateTarget(bits) {
    const exponent = ((bits & 0xff000000) >> 24) - 3;
    const mantissa = bits & 0x007fffff;
    const target = Buffer.alloc(32, 0);
    target.writeUIntBE(mantissa, 29 - exponent, 3);
    return target;
  }

  // <<<REQUIRED>>>
  static calculateMerkleRoot(transactions, forWitness) {
    typeforce([{ getHash: types.Function }], transactions);
    if (transactions.length === 0) throw errorMerkleNoTxes;
    // If the 'forWitness' flag is true and transactions don't have witness commitments, throw an error
    if (forWitness && !txesHaveWitnessCommit(transactions))
      throw errorWitnessNotSegwit;

    // Map transactions to their respective hashes using the getHash function
    const hashes = transactions.map(transaction =>
      transaction.getHash(forWitness),
    );

    // Compute the Merkle root using the fastMerkleRoot algorithm
    const rootHash = (0, merkle_1.fastMerkleRoot)(hashes, bcrypto.hash256);

    // If 'forWitness' is true, concatenate the rootHash with the witness commitment of the first transaction
    // and compute the hash of the concatenated buffer
    return forWitness
      ? bcrypto.hash256(
        Buffer.concat([rootHash, transactions[0].ins[0].witness[0]]),
      )
      : rootHash;
  }

  // <<<REQUIRED>>>
  getWitnessCommit() {
    // Check if any transactions in the block have a witness commitment
    if (!txesHaveWitnessCommit(this.transactions)) return null;
    // The merkle root for the witness data is in an OP_RETURN output.
    // There is no rule for the index of the output, so use filter to find it.
    // The root is prepended with 0xaa21a9ed so check for 0x6a24aa21a9ed
    // If multiple commits are found, the output with highest index is assumed.

    // Filter the outputs of the first transaction to find the witness commitments
    const witnessCommits = this.transactions[0].outs
      .filter(out =>
        // Check if the output script starts with a specific pattern indicating a witness commitment
        out.script.slice(0, 6).equals(Buffer.from('6a24aa21a9ed', 'hex')),
      )
      // Extract the witness commitment data from the output scripts
      .map(out => out.script.slice(6, 38));

    // If no witness commitments were found, return null
    if (witnessCommits.length === 0) return null;

    // Use the commit with the highest output (should only be one though)
    // Return the last witness commitment found (assuming multiple commitments, choose the one with the highest index)
    const result = witnessCommits[witnessCommits.length - 1];

    // Check if the result is a Buffer of 32 bytes
    if (!(result instanceof Buffer && result.length === 32)) return null;
    return result;
  }
  hasWitnessCommit() {
    if (
      this.witnessCommit instanceof Buffer &&
      this.witnessCommit.length === 32
    )
      return true;
    if (this.getWitnessCommit() !== null) return true;
    return false;
  }

  hasWitness() {
    return anyTxHasWitness(this.transactions);
  }
  weight() {
    const base = this.byteLength(false, false);
    const total = this.byteLength(false, true);
    return base * 3 + total;
  }
  byteLength(headersOnly, allowWitness = true) {
    // If only headers are requested or there are no transactions, return the fixed header size (80 bytes)
    if (headersOnly || !this.transactions) return 80;
    // Calculate the total byte length of the block including transactions
    return (
      80 +
      // Add the byte length required to encode the number of transactions (varint)
      bufferutils_1.varuint.encodingLength(this.transactions.length) +
      // Add the byte length of each transaction
      // This is calculated by iterating over each transaction and summing up their byte lengths
      this.transactions.reduce((a, x) => a + x.byteLength(allowWitness), 0)
    );
  }

  // Get the hash of the block header. It uses the hash256 function from the bcrypto module.
  getHash() {
    // Convert the block header to a buffer (excluding transactions) and hash it using SHA-256 twice.
    return bcrypto.hash256(this.toBuffer(true));
  }
  // Get the ID of the block. It reverses the hash of the block header and converts it to a hexadecimal string.
  getId() {
    // Get the hash of the block header. 
    // Reverse the hash and convert it to a hexadecimal string.
    return (0, bufferutils_1.reverseBuffer)(this.getHash()).toString('hex');
  }
  // Get the UTC date of the block based on its timestamp.
  // <<->>
  getUTCDate() {
    // Create a new Date object with epoch time (January 1, 1970 00:00:00 UTC).
    const date = new Date(0); // epoch
    // Set the seconds since epoch to the block's timestamp to calculate the date and time.
    date.setUTCSeconds(this.timestamp);
    return date;
  }
  // TODO: buffer, offset compatibility
  // <<<MAYBE REQUIRED>>>
  toBuffer(headersOnly) {
    // Allocate memory for the buffer based on the byte length of the block
    const buffer = Buffer.allocUnsafe(this.byteLength(headersOnly));
    // Create a BufferWriter instance to write data into the buffer
    const bufferWriter = new bufferutils_1.BufferWriter(buffer);

    // Write block header fields into the buffer
    bufferWriter.writeInt32(this.version);
    bufferWriter.writeSlice(this.prevHash);
    bufferWriter.writeSlice(this.merkleRoot);
    bufferWriter.writeUInt32(this.timestamp);
    bufferWriter.writeUInt32(this.bits);
    bufferWriter.writeUInt32(this.nonce);

    // If headersOnly is true or if there are no transactions, return the buffer
    if (headersOnly || !this.transactions) return buffer;

    // If there are transactions, encode the number of transactions
    bufferutils_1.varuint.encode(
      this.transactions.length, // Number of transactions
      buffer, // Buffer to write into
      bufferWriter.offset, // Offset to start writing at
    );

    // Increment the buffer offset based on the number of bytes used to encode the number of transactions
    bufferWriter.offset += bufferutils_1.varuint.encode.bytes;
    this.transactions.forEach(tx => {
      // Get the byte length of the transaction
      const txSize = tx.byteLength(); // TODO: extract from toBuffer?
      // Serialize the transaction into the buffer
      tx.toBuffer(buffer, bufferWriter.offset);
      // Increment the buffer offset by the byte length of the transaction
      bufferWriter.offset += txSize;
    });
    return buffer;
  }
  toHex(headersOnly) {
    return this.toBuffer(headersOnly).toString('hex');
  }
  // <<<REQUIRED>>>
  checkTxRoots() {
    // If the Block has segwit transactions but no witness commit,
    // there's no way it can be valid, so fail the check.
    const hasWitnessCommit = this.hasWitnessCommit(); // Check if the block has a witness commit
    if (!hasWitnessCommit && this.hasWitness()) return false; // If there are segwit transactions but no witness commit, the block is invalid
    return (
      this.__checkMerkleRoot() && // Check if the merkle root is correct
      (hasWitnessCommit ? this.__checkWitnessCommit() : true) // If there is a witness commit, check it; otherwise, return true
    );
  }
  // <<<REQUIRED>>>
  checkProofOfWork() {
    // Reverse the hash of the block
    const hash = (0, bufferutils_1.reverseBuffer)(this.getHash());
    // Calculate the target based on the block's bits
    const target = Block.calculateTarget(this.bits);
    // Check if the block's hash meets the target difficulty
    return hash.compare(target) <= 0;
  }

  // Function to check if the calculated merkle root matches the stored merkle root
  // <<<REQUIRED>>>
  __checkMerkleRoot() {
    // If there are no transactions, throw an error
    if (!this.transactions) throw errorMerkleNoTxes;

    // Calculate the actual merkle root based on the transactions
    const actualMerkleRoot = Block.calculateMerkleRoot(this.transactions);

    // Compare the calculated merkle root with the stored merkle root
    return this.merkleRoot.compare(actualMerkleRoot) === 0;
  }
  // <<<REQUIRED>>>
  __checkWitnessCommit() {
    // If there are no transactions, throw an error
    if (!this.transactions) throw errorMerkleNoTxes;

    // If the block doesn't have a witness commit, throw an error
    if (!this.hasWitnessCommit()) throw errorWitnessNotSegwit;

    // Calculate the actual witness commit based on the transactions
    const actualWitnessCommit = Block.calculateMerkleRoot(
      this.transactions,
      true,
    );

    // Compare the calculated witness commit with the stored witness commit
    return this.witnessCommit.compare(actualWitnessCommit) === 0;
  }
}
exports.Block = Block;
function txesHaveWitnessCommit(transactions) {
  return (
    transactions instanceof Array &&
    transactions[0] &&
    transactions[0].ins &&
    transactions[0].ins instanceof Array &&
    transactions[0].ins[0] &&
    transactions[0].ins[0].witness &&
    transactions[0].ins[0].witness instanceof Array &&
    transactions[0].ins[0].witness.length > 0
  );
}

// This function checks if any of the transactions in the block contain witness data.
// <<<REQUIRED>>>
function anyTxHasWitness(transactions) {
  return (
    transactions instanceof Array && // Check if transactions is an array.
    transactions.some( // Iterate over each transaction in transactions array.
      tx =>
        typeof tx === 'object' && // Ensure tx is an object.
        tx.ins instanceof Array && // Check if tx.ins is an array.
        tx.ins.some( // Iterate over each input in tx.ins array. <in our case its `Vin`>
          input =>
            typeof input === 'object' && // Ensure input is an object.
            input.witness instanceof Array && // Check if input.witness is an array.
            input.witness.length > 0, // Check if input.witness has at least one element.
        ),
    )
  );
}
