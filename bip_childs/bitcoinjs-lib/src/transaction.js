'use strict';
Object.defineProperty(exports, '__esModule', { value: true });
exports.Transaction = void 0;
const bufferutils_1 = require('./bufferutils');
const bcrypto = require('./crypto');
const bscript = require('./script');
const script_1 = require('./script');
const types = require('./types');
const { typeforce } = types;

// Function to calculate the size of a variable-length slice
function varSliceSize(someScript) {
  const length = someScript.length;
  // Calculate the size by adding the length of the script data to the length of its encoding
  return bufferutils_1.varuint.encodingLength(length) + length;
}

// Function to calculate the size of a vector (array of variable-length slices)
function vectorSize(someVector) {
  const length = someVector.length;
  // Calculate the size by adding the encoding length of the vector length
  // and the size of each element in the vector
  return (
    bufferutils_1.varuint.encodingLength(length) +
    someVector.reduce((sum, witness) => {
      return sum + varSliceSize(witness);
    }, 0)
  );
}
const EMPTY_BUFFER = Buffer.allocUnsafe(0);
const EMPTY_WITNESS = [];
const ZERO = Buffer.from(
  '0000000000000000000000000000000000000000000000000000000000000000',
  'hex',
);
const ONE = Buffer.from(
  '0000000000000000000000000000000000000000000000000000000000000001',
  'hex',
);
const VALUE_UINT64_MAX = Buffer.from('ffffffffffffffff', 'hex');
const BLANK_OUTPUT = {
  script: EMPTY_BUFFER,
  valueBuffer: VALUE_UINT64_MAX,
};

// Function to check if an output is defined (has a value)
function isOutput(out) {
  return out.value !== undefined;
}
/**
 * Represents a Bitcoin transaction.
 */
class Transaction {
  constructor() {
    this.version = 1;
    this.locktime = 0;
    this.ins = [];
    this.outs = [];
  }
  // <<<REQUIRED>>>
  static fromBuffer(buffer, _NO_STRICT) {
    // Create a buffer reader to read from the provided buffer
    const bufferReader = new bufferutils_1.BufferReader(buffer);
    // Create a new Transaction object
    const tx = new Transaction();
    // Read the transaction version
    tx.version = bufferReader.readInt32();
    // Read the marker and flag bytes <<WHAT IS THE USE>>
    const marker = bufferReader.readUInt8();
    const flag = bufferReader.readUInt8();

    // Check if the transaction has witness data
    let hasWitnesses = false;
    if (
      marker === Transaction.ADVANCED_TRANSACTION_MARKER &&
      flag === Transaction.ADVANCED_TRANSACTION_FLAG
    ) {
      hasWitnesses = true;
    } else {
      // If not, adjust the bufferReader offset to skip the marker and flag bytes <<WHY??>>
      bufferReader.offset -= 2;
    }

    // Read the number of transaction inputs (vinLen)
    const vinLen = bufferReader.readVarInt();
    for (let i = 0; i < vinLen; ++i) {
      tx.ins.push({
        hash: bufferReader.readSlice(32), // Read input hash (32 bytes)
        index: bufferReader.readUInt32(), // Read input index (4 bytes)
        script: bufferReader.readVarSlice(), // Read input script (variable length)
        sequence: bufferReader.readUInt32(), // Read input sequence number (4 bytes)
        witness: EMPTY_WITNESS, // Initialize input witness as an empty array
      });
    }

    // Read the number of transaction outputs (voutLen)
    const voutLen = bufferReader.readVarInt();
    for (let i = 0; i < voutLen; ++i) {
      tx.outs.push({
        value: bufferReader.readUInt64(), // Read output value (8 bytes)
        script: bufferReader.readVarSlice(), // Read output script (variable length)
      });
    }
    if (hasWitnesses) { // If the transaction has witnesses, read witness data for each input
      for (let i = 0; i < vinLen; ++i) {
        tx.ins[i].witness = bufferReader.readVector();
      }
      // was this pointless?
      if (!tx.hasWitnesses())
        throw new Error('Transaction has superfluous witness data');
    }

    // Read the transaction locktime
    tx.locktime = bufferReader.readUInt32();

    // NOT_REQUIRED
    if (_NO_STRICT) return tx;
    if (bufferReader.offset !== buffer.length)
      throw new Error('Transaction has unexpected data');
    return tx;
  }
  static fromHex(hex) {
    return Transaction.fromBuffer(Buffer.from(hex, 'hex'), false);
  }
  // <<<REQUIRED>>>
  static isCoinbaseHash(buffer) {
    // Ensure the buffer has the correct length (32 bytes)
    typeforce(types.Hash256bit, buffer);
    // Check if the buffer consists of all zeros
    for (let i = 0; i < 32; ++i) {
      if (buffer[i] !== 0) return false; // If any byte is non-zero, return false
    }
    return true; // If all bytes are zero, return true
  }
  // <<<REQUIRED>>>
  isCoinbase() {
    // Check if the transaction is a coinbase transaction
    // A coinbase transaction has only one input and its hash is all zeros
    return (
      this.ins.length === 1 && Transaction.isCoinbaseHash(this.ins[0].hash)
    );
  }
  // <<<NOT__REQUIRED>>>
  addInput(hash, index, sequence, scriptSig) {
    typeforce(
      types.tuple(
        types.Hash256bit,
        types.UInt32,
        types.maybe(types.UInt32),
        types.maybe(types.Buffer),
      ),
      arguments,
    );

    // If sequence is not provided, default to Transaction.DEFAULT_SEQUENCE
    if (types.Null(sequence)) {
      sequence = Transaction.DEFAULT_SEQUENCE;
    }
    // Add the input and return the input's index
    return (
      this.ins.push({
        hash,
        index,
        script: scriptSig || EMPTY_BUFFER, // Use provided scriptSig or an empty buffer
        sequence: sequence,
        witness: EMPTY_WITNESS, // Initialize witness data to an empty array
      }) - 1
    );
  }
  // <<<NOT__REQUIRED>>>
  addOutput(scriptPubKey, value) {
    typeforce(types.tuple(types.Buffer, types.Satoshi), arguments);
    // Add the output and return the output's index
    return (
      this.outs.push({
        script: scriptPubKey,
        value,
      }) - 1
    );
  }
  // <<<REQUIRED>>>
  hasWitnesses() {
    return this.ins.some(x => {
      return x.witness.length !== 0;
    });
  }
  weight() {
    const base = this.byteLength(false);
    const total = this.byteLength(true);
    return base * 3 + total;
  }
  virtualSize() {
    return Math.ceil(this.weight() / 4);
  }
  byteLength(_ALLOW_WITNESS = true) {
    const hasWitnesses = _ALLOW_WITNESS && this.hasWitnesses();
    return (
      (hasWitnesses ? 10 : 8) +
      bufferutils_1.varuint.encodingLength(this.ins.length) +
      bufferutils_1.varuint.encodingLength(this.outs.length) +
      this.ins.reduce((sum, input) => {
        return sum + 40 + varSliceSize(input.script);
      }, 0) +
      this.outs.reduce((sum, output) => {
        return sum + 8 + varSliceSize(output.script);
      }, 0) +
      (hasWitnesses
        ? this.ins.reduce((sum, input) => {
          return sum + vectorSize(input.witness);
        }, 0)
        : 0)
    );
  }
  clone() {
    const newTx = new Transaction();
    newTx.version = this.version;
    newTx.locktime = this.locktime;
    newTx.ins = this.ins.map(txIn => {
      return {
        hash: txIn.hash,
        index: txIn.index,
        script: txIn.script,
        sequence: txIn.sequence,
        witness: txIn.witness,
      };
    });
    newTx.outs = this.outs.map(txOut => {
      return {
        script: txOut.script,
        value: txOut.value,
      };
    });
    return newTx;
  }
  /**
   * Hash transaction for signing a specific input.
   *
   * Bitcoin uses a different hash for each signed transaction input.
   * This method copies the transaction, makes the necessary changes based on the
   * hashType, and then hashes the result.
   * This hash can then be used to sign the provided transaction input.
   */
  hashForSignature(inIndex, prevOutScript, hashType) {
    typeforce(
      types.tuple(types.UInt32, types.Buffer, /* types.UInt8 */ types.Number),
      arguments,
    );
    // https://github.com/bitcoin/bitcoin/blob/master/src/test/sighash_tests.cpp#L29
    if (inIndex >= this.ins.length) return ONE;
    // ignore OP_CODESEPARATOR
    const ourScript = bscript.compile(
      bscript.decompile(prevOutScript).filter(x => {
        return x !== script_1.OPS.OP_CODESEPARATOR;
      }),
    );
    const txTmp = this.clone();
    // SIGHASH_NONE: ignore all outputs? (wildcard payee)
    if ((hashType & 0x1f) === Transaction.SIGHASH_NONE) {
      txTmp.outs = [];
      // ignore sequence numbers (except at inIndex)
      txTmp.ins.forEach((input, i) => {
        if (i === inIndex) return;
        input.sequence = 0;
      });
      // SIGHASH_SINGLE: ignore all outputs, except at the same index?
    } else if ((hashType & 0x1f) === Transaction.SIGHASH_SINGLE) {
      // https://github.com/bitcoin/bitcoin/blob/master/src/test/sighash_tests.cpp#L60
      if (inIndex >= this.outs.length) return ONE;
      // truncate outputs after
      txTmp.outs.length = inIndex + 1;
      // "blank" outputs before
      for (let i = 0; i < inIndex; i++) {
        txTmp.outs[i] = BLANK_OUTPUT;
      }
      // ignore sequence numbers (except at inIndex)
      txTmp.ins.forEach((input, y) => {
        if (y === inIndex) return;
        input.sequence = 0;
      });
    }
    // SIGHASH_ANYONECANPAY: ignore inputs entirely?
    if (hashType & Transaction.SIGHASH_ANYONECANPAY) {
      txTmp.ins = [txTmp.ins[inIndex]];
      txTmp.ins[0].script = ourScript;
      // SIGHASH_ALL: only ignore input scripts
    } else {
      // "blank" others input scripts
      txTmp.ins.forEach(input => {
        input.script = EMPTY_BUFFER;
      });
      txTmp.ins[inIndex].script = ourScript;
    }
    // serialize and hash
    const buffer = Buffer.allocUnsafe(txTmp.byteLength(false) + 4);
    buffer.writeInt32LE(hashType, buffer.length - 4);
    txTmp.__toBuffer(buffer, 0, false);
    return bcrypto.hash256(buffer);
  }
  hashForWitnessV1(inIndex, prevOutScripts, values, hashType, leafHash, annex) {
    // https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#common-signature-message
    typeforce(
      types.tuple(
        types.UInt32,
        typeforce.arrayOf(types.Buffer),
        typeforce.arrayOf(types.Satoshi),
        types.UInt32,
      ),
      arguments,
    );
    if (
      values.length !== this.ins.length ||
      prevOutScripts.length !== this.ins.length
    ) {
      throw new Error('Must supply prevout script and value for all inputs');
    }
    const outputType =
      hashType === Transaction.SIGHASH_DEFAULT
        ? Transaction.SIGHASH_ALL
        : hashType & Transaction.SIGHASH_OUTPUT_MASK;
    const inputType = hashType & Transaction.SIGHASH_INPUT_MASK;
    const isAnyoneCanPay = inputType === Transaction.SIGHASH_ANYONECANPAY;
    const isNone = outputType === Transaction.SIGHASH_NONE;
    const isSingle = outputType === Transaction.SIGHASH_SINGLE;
    let hashPrevouts = EMPTY_BUFFER;
    let hashAmounts = EMPTY_BUFFER;
    let hashScriptPubKeys = EMPTY_BUFFER;
    let hashSequences = EMPTY_BUFFER;
    let hashOutputs = EMPTY_BUFFER;
    if (!isAnyoneCanPay) {
      let bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        36 * this.ins.length,
      );
      this.ins.forEach(txIn => {
        bufferWriter.writeSlice(txIn.hash);
        bufferWriter.writeUInt32(txIn.index);
      });
      hashPrevouts = bcrypto.sha256(bufferWriter.end());
      bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        8 * this.ins.length,
      );
      values.forEach(value => bufferWriter.writeUInt64(value));
      hashAmounts = bcrypto.sha256(bufferWriter.end());
      bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        prevOutScripts.map(varSliceSize).reduce((a, b) => a + b),
      );
      prevOutScripts.forEach(prevOutScript =>
        bufferWriter.writeVarSlice(prevOutScript),
      );
      hashScriptPubKeys = bcrypto.sha256(bufferWriter.end());
      bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        4 * this.ins.length,
      );
      this.ins.forEach(txIn => bufferWriter.writeUInt32(txIn.sequence));
      hashSequences = bcrypto.sha256(bufferWriter.end());
    }
    if (!(isNone || isSingle)) {
      const txOutsSize = this.outs
        .map(output => 8 + varSliceSize(output.script))
        .reduce((a, b) => a + b);
      const bufferWriter = bufferutils_1.BufferWriter.withCapacity(txOutsSize);
      this.outs.forEach(out => {
        bufferWriter.writeUInt64(out.value);
        bufferWriter.writeVarSlice(out.script);
      });
      hashOutputs = bcrypto.sha256(bufferWriter.end());
    } else if (isSingle && inIndex < this.outs.length) {
      const output = this.outs[inIndex];
      const bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        8 + varSliceSize(output.script),
      );
      bufferWriter.writeUInt64(output.value);
      bufferWriter.writeVarSlice(output.script);
      hashOutputs = bcrypto.sha256(bufferWriter.end());
    }
    const spendType = (leafHash ? 2 : 0) + (annex ? 1 : 0);
    // Length calculation from:
    // https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#cite_note-14
    // With extension from:
    // https://github.com/bitcoin/bips/blob/master/bip-0342.mediawiki#signature-validation
    const sigMsgSize =
      174 -
      (isAnyoneCanPay ? 49 : 0) -
      (isNone ? 32 : 0) +
      (annex ? 32 : 0) +
      (leafHash ? 37 : 0);
    const sigMsgWriter = bufferutils_1.BufferWriter.withCapacity(sigMsgSize);
    sigMsgWriter.writeUInt8(hashType);
    // Transaction
    sigMsgWriter.writeInt32(this.version);
    sigMsgWriter.writeUInt32(this.locktime);
    sigMsgWriter.writeSlice(hashPrevouts);
    sigMsgWriter.writeSlice(hashAmounts);
    sigMsgWriter.writeSlice(hashScriptPubKeys);
    sigMsgWriter.writeSlice(hashSequences);
    if (!(isNone || isSingle)) {
      sigMsgWriter.writeSlice(hashOutputs);
    }
    // Input
    sigMsgWriter.writeUInt8(spendType);
    if (isAnyoneCanPay) {
      const input = this.ins[inIndex];
      sigMsgWriter.writeSlice(input.hash);
      sigMsgWriter.writeUInt32(input.index);
      sigMsgWriter.writeUInt64(values[inIndex]);
      sigMsgWriter.writeVarSlice(prevOutScripts[inIndex]);
      sigMsgWriter.writeUInt32(input.sequence);
    } else {
      sigMsgWriter.writeUInt32(inIndex);
    }
    if (annex) {
      const bufferWriter = bufferutils_1.BufferWriter.withCapacity(
        varSliceSize(annex),
      );
      bufferWriter.writeVarSlice(annex);
      sigMsgWriter.writeSlice(bcrypto.sha256(bufferWriter.end()));
    }
    // Output
    if (isSingle) {
      sigMsgWriter.writeSlice(hashOutputs);
    }
    // BIP342 extension
    if (leafHash) {
      sigMsgWriter.writeSlice(leafHash);
      sigMsgWriter.writeUInt8(0);
      sigMsgWriter.writeUInt32(0xffffffff);
    }
    // Extra zero byte because:
    // https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#cite_note-19
    return bcrypto.taggedHash(
      'TapSighash',
      Buffer.concat([Buffer.from([0x00]), sigMsgWriter.end()]),
    );
  }
  hashForWitnessV0(inIndex, prevOutScript, value, hashType) {
    typeforce(
      types.tuple(types.UInt32, types.Buffer, types.Satoshi, types.UInt32),
      arguments,
    );
    let tbuffer = Buffer.from([]);
    let bufferWriter;
    let hashOutputs = ZERO;
    let hashPrevouts = ZERO;
    let hashSequence = ZERO;
    if (!(hashType & Transaction.SIGHASH_ANYONECANPAY)) {
      tbuffer = Buffer.allocUnsafe(36 * this.ins.length);
      bufferWriter = new bufferutils_1.BufferWriter(tbuffer, 0);
      this.ins.forEach(txIn => {
        bufferWriter.writeSlice(txIn.hash);
        bufferWriter.writeUInt32(txIn.index);
      });
      hashPrevouts = bcrypto.hash256(tbuffer);
    }
    if (
      !(hashType & Transaction.SIGHASH_ANYONECANPAY) &&
      (hashType & 0x1f) !== Transaction.SIGHASH_SINGLE &&
      (hashType & 0x1f) !== Transaction.SIGHASH_NONE
    ) {
      tbuffer = Buffer.allocUnsafe(4 * this.ins.length);
      bufferWriter = new bufferutils_1.BufferWriter(tbuffer, 0);
      this.ins.forEach(txIn => {
        bufferWriter.writeUInt32(txIn.sequence);
      });
      hashSequence = bcrypto.hash256(tbuffer);
    }
    if (
      (hashType & 0x1f) !== Transaction.SIGHASH_SINGLE &&
      (hashType & 0x1f) !== Transaction.SIGHASH_NONE
    ) {
      const txOutsSize = this.outs.reduce((sum, output) => {
        return sum + 8 + varSliceSize(output.script);
      }, 0);
      tbuffer = Buffer.allocUnsafe(txOutsSize);
      bufferWriter = new bufferutils_1.BufferWriter(tbuffer, 0);
      this.outs.forEach(out => {
        bufferWriter.writeUInt64(out.value);
        bufferWriter.writeVarSlice(out.script);
      });
      hashOutputs = bcrypto.hash256(tbuffer);
    } else if (
      (hashType & 0x1f) === Transaction.SIGHASH_SINGLE &&
      inIndex < this.outs.length
    ) {
      const output = this.outs[inIndex];
      tbuffer = Buffer.allocUnsafe(8 + varSliceSize(output.script));
      bufferWriter = new bufferutils_1.BufferWriter(tbuffer, 0);
      bufferWriter.writeUInt64(output.value);
      bufferWriter.writeVarSlice(output.script);
      hashOutputs = bcrypto.hash256(tbuffer);
    }
    tbuffer = Buffer.allocUnsafe(156 + varSliceSize(prevOutScript));
    bufferWriter = new bufferutils_1.BufferWriter(tbuffer, 0);
    const input = this.ins[inIndex];
    bufferWriter.writeInt32(this.version);
    bufferWriter.writeSlice(hashPrevouts);
    bufferWriter.writeSlice(hashSequence);
    bufferWriter.writeSlice(input.hash);
    bufferWriter.writeUInt32(input.index);
    bufferWriter.writeVarSlice(prevOutScript);
    bufferWriter.writeUInt64(value);
    bufferWriter.writeUInt32(input.sequence);
    bufferWriter.writeSlice(hashOutputs);
    bufferWriter.writeUInt32(this.locktime);
    bufferWriter.writeUInt32(hashType);
    return bcrypto.hash256(tbuffer);
  }
  getHash(forWitness) {
    // wtxid for coinbase is always 32 bytes of 0x00
    if (forWitness && this.isCoinbase()) return Buffer.alloc(32, 0);
    return bcrypto.hash256(this.__toBuffer(undefined, undefined, forWitness));
  }
  getId() {
    // transaction hash's are displayed in reverse order
    return (0, bufferutils_1.reverseBuffer)(this.getHash(false)).toString(
      'hex',
    );
  }
  toBuffer(buffer, initialOffset) {
    return this.__toBuffer(buffer, initialOffset, true);
  }
  toHex() {
    return this.toBuffer(undefined, undefined).toString('hex');
  }
  setInputScript(index, scriptSig) {
    typeforce(types.tuple(types.Number, types.Buffer), arguments);
    this.ins[index].script = scriptSig;
  }
  setWitness(index, witness) {
    typeforce(types.tuple(types.Number, [types.Buffer]), arguments);
    this.ins[index].witness = witness;
  }
  __toBuffer(buffer, initialOffset, _ALLOW_WITNESS = false) {
    if (!buffer) buffer = Buffer.allocUnsafe(this.byteLength(_ALLOW_WITNESS));
    const bufferWriter = new bufferutils_1.BufferWriter(
      buffer,
      initialOffset || 0,
    );
    bufferWriter.writeInt32(this.version);
    const hasWitnesses = _ALLOW_WITNESS && this.hasWitnesses();
    if (hasWitnesses) {
      bufferWriter.writeUInt8(Transaction.ADVANCED_TRANSACTION_MARKER);
      bufferWriter.writeUInt8(Transaction.ADVANCED_TRANSACTION_FLAG);
    }
    bufferWriter.writeVarInt(this.ins.length);
    this.ins.forEach(txIn => {
      bufferWriter.writeSlice(txIn.hash);
      bufferWriter.writeUInt32(txIn.index);
      bufferWriter.writeVarSlice(txIn.script);
      bufferWriter.writeUInt32(txIn.sequence);
    });
    bufferWriter.writeVarInt(this.outs.length);
    this.outs.forEach(txOut => {
      if (isOutput(txOut)) {
        bufferWriter.writeUInt64(txOut.value);
      } else {
        bufferWriter.writeSlice(txOut.valueBuffer);
      }
      bufferWriter.writeVarSlice(txOut.script);
    });
    if (hasWitnesses) {
      this.ins.forEach(input => {
        bufferWriter.writeVector(input.witness);
      });
    }
    bufferWriter.writeUInt32(this.locktime);
    // avoid slicing unless necessary
    if (initialOffset !== undefined)
      return buffer.slice(initialOffset, bufferWriter.offset);
    return buffer;
  }
}
exports.Transaction = Transaction;
Transaction.DEFAULT_SEQUENCE = 0xffffffff;
Transaction.SIGHASH_DEFAULT = 0x00;
Transaction.SIGHASH_ALL = 0x01;
Transaction.SIGHASH_NONE = 0x02;
Transaction.SIGHASH_SINGLE = 0x03;
Transaction.SIGHASH_ANYONECANPAY = 0x80;
Transaction.SIGHASH_OUTPUT_MASK = 0x03;
Transaction.SIGHASH_INPUT_MASK = 0x80;
Transaction.ADVANCED_TRANSACTION_MARKER = 0x00;
Transaction.ADVANCED_TRANSACTION_FLAG = 0x01;
