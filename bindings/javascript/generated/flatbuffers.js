var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/constants.js
var require_constants = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/constants.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.SIZE_PREFIX_LENGTH = exports.FILE_IDENTIFIER_LENGTH = exports.SIZEOF_INT = exports.SIZEOF_SHORT = void 0;
    exports.SIZEOF_SHORT = 2;
    exports.SIZEOF_INT = 4;
    exports.FILE_IDENTIFIER_LENGTH = 4;
    exports.SIZE_PREFIX_LENGTH = 4;
  }
});

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/utils.js
var require_utils = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/utils.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.isLittleEndian = exports.float64 = exports.float32 = exports.int32 = void 0;
    exports.int32 = new Int32Array(2);
    exports.float32 = new Float32Array(exports.int32.buffer);
    exports.float64 = new Float64Array(exports.int32.buffer);
    exports.isLittleEndian = new Uint16Array(new Uint8Array([1, 0]).buffer)[0] === 1;
  }
});

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/encoding.js
var require_encoding = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/encoding.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Encoding = void 0;
    var Encoding;
    (function(Encoding2) {
      Encoding2[Encoding2["UTF8_BYTES"] = 1] = "UTF8_BYTES";
      Encoding2[Encoding2["UTF16_STRING"] = 2] = "UTF16_STRING";
    })(Encoding || (exports.Encoding = Encoding = {}));
  }
});

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/byte-buffer.js
var require_byte_buffer = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/byte-buffer.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.ByteBuffer = void 0;
    var constants_js_1 = require_constants();
    var encoding_js_1 = require_encoding();
    var utils_js_1 = require_utils();
    var ByteBuffer = class _ByteBuffer {
      /**
       * Create a new ByteBuffer with a given array of bytes (`Uint8Array`)
       */
      constructor(bytes_) {
        this.bytes_ = bytes_;
        this.position_ = 0;
        this.text_decoder_ = new TextDecoder();
      }
      /**
       * Create and allocate a new ByteBuffer with a given size.
       */
      static allocate(byte_size) {
        return new _ByteBuffer(new Uint8Array(byte_size));
      }
      clear() {
        this.position_ = 0;
      }
      /**
       * Get the underlying `Uint8Array`.
       */
      bytes() {
        return this.bytes_;
      }
      /**
       * Get the buffer's position.
       */
      position() {
        return this.position_;
      }
      /**
       * Set the buffer's position.
       */
      setPosition(position) {
        this.position_ = position;
      }
      /**
       * Get the buffer's capacity.
       */
      capacity() {
        return this.bytes_.length;
      }
      readInt8(offset) {
        return this.readUint8(offset) << 24 >> 24;
      }
      readUint8(offset) {
        return this.bytes_[offset];
      }
      readInt16(offset) {
        return this.readUint16(offset) << 16 >> 16;
      }
      readUint16(offset) {
        return this.bytes_[offset] | this.bytes_[offset + 1] << 8;
      }
      readInt32(offset) {
        return this.bytes_[offset] | this.bytes_[offset + 1] << 8 | this.bytes_[offset + 2] << 16 | this.bytes_[offset + 3] << 24;
      }
      readUint32(offset) {
        return this.readInt32(offset) >>> 0;
      }
      readInt64(offset) {
        return BigInt.asIntN(64, BigInt(this.readUint32(offset)) + (BigInt(this.readUint32(offset + 4)) << BigInt(32)));
      }
      readUint64(offset) {
        return BigInt.asUintN(64, BigInt(this.readUint32(offset)) + (BigInt(this.readUint32(offset + 4)) << BigInt(32)));
      }
      readFloat32(offset) {
        utils_js_1.int32[0] = this.readInt32(offset);
        return utils_js_1.float32[0];
      }
      readFloat64(offset) {
        utils_js_1.int32[utils_js_1.isLittleEndian ? 0 : 1] = this.readInt32(offset);
        utils_js_1.int32[utils_js_1.isLittleEndian ? 1 : 0] = this.readInt32(offset + 4);
        return utils_js_1.float64[0];
      }
      writeInt8(offset, value) {
        this.bytes_[offset] = value;
      }
      writeUint8(offset, value) {
        this.bytes_[offset] = value;
      }
      writeInt16(offset, value) {
        this.bytes_[offset] = value;
        this.bytes_[offset + 1] = value >> 8;
      }
      writeUint16(offset, value) {
        this.bytes_[offset] = value;
        this.bytes_[offset + 1] = value >> 8;
      }
      writeInt32(offset, value) {
        this.bytes_[offset] = value;
        this.bytes_[offset + 1] = value >> 8;
        this.bytes_[offset + 2] = value >> 16;
        this.bytes_[offset + 3] = value >> 24;
      }
      writeUint32(offset, value) {
        this.bytes_[offset] = value;
        this.bytes_[offset + 1] = value >> 8;
        this.bytes_[offset + 2] = value >> 16;
        this.bytes_[offset + 3] = value >> 24;
      }
      writeInt64(offset, value) {
        this.writeInt32(offset, Number(BigInt.asIntN(32, value)));
        this.writeInt32(offset + 4, Number(BigInt.asIntN(32, value >> BigInt(32))));
      }
      writeUint64(offset, value) {
        this.writeUint32(offset, Number(BigInt.asUintN(32, value)));
        this.writeUint32(offset + 4, Number(BigInt.asUintN(32, value >> BigInt(32))));
      }
      writeFloat32(offset, value) {
        utils_js_1.float32[0] = value;
        this.writeInt32(offset, utils_js_1.int32[0]);
      }
      writeFloat64(offset, value) {
        utils_js_1.float64[0] = value;
        this.writeInt32(offset, utils_js_1.int32[utils_js_1.isLittleEndian ? 0 : 1]);
        this.writeInt32(offset + 4, utils_js_1.int32[utils_js_1.isLittleEndian ? 1 : 0]);
      }
      /**
       * Return the file identifier.   Behavior is undefined for FlatBuffers whose
       * schema does not include a file_identifier (likely points at padding or the
       * start of a the root vtable).
       */
      getBufferIdentifier() {
        if (this.bytes_.length < this.position_ + constants_js_1.SIZEOF_INT + constants_js_1.FILE_IDENTIFIER_LENGTH) {
          throw new Error("FlatBuffers: ByteBuffer is too short to contain an identifier.");
        }
        let result = "";
        for (let i = 0; i < constants_js_1.FILE_IDENTIFIER_LENGTH; i++) {
          result += String.fromCharCode(this.readInt8(this.position_ + constants_js_1.SIZEOF_INT + i));
        }
        return result;
      }
      /**
       * Look up a field in the vtable, return an offset into the object, or 0 if the
       * field is not present.
       */
      __offset(bb_pos, vtable_offset) {
        const vtable = bb_pos - this.readInt32(bb_pos);
        return vtable_offset < this.readInt16(vtable) ? this.readInt16(vtable + vtable_offset) : 0;
      }
      /**
       * Initialize any Table-derived type to point to the union at the given offset.
       */
      __union(t, offset) {
        t.bb_pos = offset + this.readInt32(offset);
        t.bb = this;
        return t;
      }
      /**
       * Create a JavaScript string from UTF-8 data stored inside the FlatBuffer.
       * This allocates a new string and converts to wide chars upon each access.
       *
       * To avoid the conversion to string, pass Encoding.UTF8_BYTES as the
       * "optionalEncoding" argument. This is useful for avoiding conversion when
       * the data will just be packaged back up in another FlatBuffer later on.
       *
       * @param offset
       * @param opt_encoding Defaults to UTF16_STRING
       */
      __string(offset, opt_encoding) {
        offset += this.readInt32(offset);
        const length = this.readInt32(offset);
        offset += constants_js_1.SIZEOF_INT;
        const utf8bytes = this.bytes_.subarray(offset, offset + length);
        if (opt_encoding === encoding_js_1.Encoding.UTF8_BYTES)
          return utf8bytes;
        else
          return this.text_decoder_.decode(utf8bytes);
      }
      /**
       * Handle unions that can contain string as its member, if a Table-derived type then initialize it,
       * if a string then return a new one
       *
       * WARNING: strings are immutable in JS so we can't change the string that the user gave us, this
       * makes the behaviour of __union_with_string different compared to __union
       */
      __union_with_string(o, offset) {
        if (typeof o === "string") {
          return this.__string(offset);
        }
        return this.__union(o, offset);
      }
      /**
       * Retrieve the relative offset stored at "offset"
       */
      __indirect(offset) {
        return offset + this.readInt32(offset);
      }
      /**
       * Get the start of data of a vector whose offset is stored at "offset" in this object.
       */
      __vector(offset) {
        return offset + this.readInt32(offset) + constants_js_1.SIZEOF_INT;
      }
      /**
       * Get the length of a vector whose offset is stored at "offset" in this object.
       */
      __vector_len(offset) {
        return this.readInt32(offset + this.readInt32(offset));
      }
      __has_identifier(ident) {
        if (ident.length != constants_js_1.FILE_IDENTIFIER_LENGTH) {
          throw new Error("FlatBuffers: file identifier must be length " + constants_js_1.FILE_IDENTIFIER_LENGTH);
        }
        for (let i = 0; i < constants_js_1.FILE_IDENTIFIER_LENGTH; i++) {
          if (ident.charCodeAt(i) != this.readInt8(this.position() + constants_js_1.SIZEOF_INT + i)) {
            return false;
          }
        }
        return true;
      }
      /**
       * A helper function for generating list for obj api
       */
      createScalarList(listAccessor, listLength) {
        const ret = [];
        for (let i = 0; i < listLength; ++i) {
          const val = listAccessor(i);
          if (val !== null) {
            ret.push(val);
          }
        }
        return ret;
      }
      /**
       * A helper function for generating list for obj api
       * @param listAccessor function that accepts an index and return data at that index
       * @param listLength listLength
       * @param res result list
       */
      createObjList(listAccessor, listLength) {
        const ret = [];
        for (let i = 0; i < listLength; ++i) {
          const val = listAccessor(i);
          if (val !== null) {
            ret.push(val.unpack());
          }
        }
        return ret;
      }
    };
    exports.ByteBuffer = ByteBuffer;
  }
});

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/builder.js
var require_builder = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/builder.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Builder = void 0;
    var byte_buffer_js_1 = require_byte_buffer();
    var constants_js_1 = require_constants();
    var Builder = class _Builder {
      /**
       * Create a FlatBufferBuilder.
       */
      constructor(opt_initial_size) {
        this.minalign = 1;
        this.vtable = null;
        this.vtable_in_use = 0;
        this.isNested = false;
        this.object_start = 0;
        this.vtables = [];
        this.vector_num_elems = 0;
        this.force_defaults = false;
        this.string_maps = null;
        this.text_encoder = new TextEncoder();
        let initial_size;
        if (!opt_initial_size) {
          initial_size = 1024;
        } else {
          initial_size = opt_initial_size;
        }
        this.bb = byte_buffer_js_1.ByteBuffer.allocate(initial_size);
        this.space = initial_size;
      }
      clear() {
        this.bb.clear();
        this.space = this.bb.capacity();
        this.minalign = 1;
        this.vtable = null;
        this.vtable_in_use = 0;
        this.isNested = false;
        this.object_start = 0;
        this.vtables = [];
        this.vector_num_elems = 0;
        this.force_defaults = false;
        this.string_maps = null;
      }
      /**
       * In order to save space, fields that are set to their default value
       * don't get serialized into the buffer. Forcing defaults provides a
       * way to manually disable this optimization.
       *
       * @param forceDefaults true always serializes default values
       */
      forceDefaults(forceDefaults) {
        this.force_defaults = forceDefaults;
      }
      /**
       * Get the ByteBuffer representing the FlatBuffer. Only call this after you've
       * called finish(). The actual data starts at the ByteBuffer's current position,
       * not necessarily at 0.
       */
      dataBuffer() {
        return this.bb;
      }
      /**
       * Get the bytes representing the FlatBuffer. Only call this after you've
       * called finish().
       */
      asUint8Array() {
        return this.bb.bytes().subarray(this.bb.position(), this.bb.position() + this.offset());
      }
      /**
       * Prepare to write an element of `size` after `additional_bytes` have been
       * written, e.g. if you write a string, you need to align such the int length
       * field is aligned to 4 bytes, and the string data follows it directly. If all
       * you need to do is alignment, `additional_bytes` will be 0.
       *
       * @param size This is the of the new element to write
       * @param additional_bytes The padding size
       */
      prep(size, additional_bytes) {
        if (size > this.minalign) {
          this.minalign = size;
        }
        const align_size = ~(this.bb.capacity() - this.space + additional_bytes) + 1 & size - 1;
        while (this.space < align_size + size + additional_bytes) {
          const old_buf_size = this.bb.capacity();
          this.bb = _Builder.growByteBuffer(this.bb);
          this.space += this.bb.capacity() - old_buf_size;
        }
        this.pad(align_size);
      }
      pad(byte_size) {
        for (let i = 0; i < byte_size; i++) {
          this.bb.writeInt8(--this.space, 0);
        }
      }
      writeInt8(value) {
        this.bb.writeInt8(this.space -= 1, value);
      }
      writeInt16(value) {
        this.bb.writeInt16(this.space -= 2, value);
      }
      writeInt32(value) {
        this.bb.writeInt32(this.space -= 4, value);
      }
      writeInt64(value) {
        this.bb.writeInt64(this.space -= 8, value);
      }
      writeFloat32(value) {
        this.bb.writeFloat32(this.space -= 4, value);
      }
      writeFloat64(value) {
        this.bb.writeFloat64(this.space -= 8, value);
      }
      /**
       * Add an `int8` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `int8` to add the buffer.
       */
      addInt8(value) {
        this.prep(1, 0);
        this.writeInt8(value);
      }
      /**
       * Add an `int16` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `int16` to add the buffer.
       */
      addInt16(value) {
        this.prep(2, 0);
        this.writeInt16(value);
      }
      /**
       * Add an `int32` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `int32` to add the buffer.
       */
      addInt32(value) {
        this.prep(4, 0);
        this.writeInt32(value);
      }
      /**
       * Add an `int64` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `int64` to add the buffer.
       */
      addInt64(value) {
        this.prep(8, 0);
        this.writeInt64(value);
      }
      /**
       * Add a `float32` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `float32` to add the buffer.
       */
      addFloat32(value) {
        this.prep(4, 0);
        this.writeFloat32(value);
      }
      /**
       * Add a `float64` to the buffer, properly aligned, and grows the buffer (if necessary).
       * @param value The `float64` to add the buffer.
       */
      addFloat64(value) {
        this.prep(8, 0);
        this.writeFloat64(value);
      }
      addFieldInt8(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addInt8(value);
          this.slot(voffset);
        }
      }
      addFieldInt16(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addInt16(value);
          this.slot(voffset);
        }
      }
      addFieldInt32(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addInt32(value);
          this.slot(voffset);
        }
      }
      addFieldInt64(voffset, value, defaultValue) {
        if (this.force_defaults || value !== defaultValue) {
          this.addInt64(value);
          this.slot(voffset);
        }
      }
      addFieldFloat32(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addFloat32(value);
          this.slot(voffset);
        }
      }
      addFieldFloat64(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addFloat64(value);
          this.slot(voffset);
        }
      }
      addFieldOffset(voffset, value, defaultValue) {
        if (this.force_defaults || value != defaultValue) {
          this.addOffset(value);
          this.slot(voffset);
        }
      }
      /**
       * Structs are stored inline, so nothing additional is being added. `d` is always 0.
       */
      addFieldStruct(voffset, value, defaultValue) {
        if (value != defaultValue) {
          this.nested(value);
          this.slot(voffset);
        }
      }
      /**
       * Structures are always stored inline, they need to be created right
       * where they're used.  You'll get this assertion failure if you
       * created it elsewhere.
       */
      nested(obj) {
        if (obj != this.offset()) {
          throw new TypeError("FlatBuffers: struct must be serialized inline.");
        }
      }
      /**
       * Should not be creating any other object, string or vector
       * while an object is being constructed
       */
      notNested() {
        if (this.isNested) {
          throw new TypeError("FlatBuffers: object serialization must not be nested.");
        }
      }
      /**
       * Set the current vtable at `voffset` to the current location in the buffer.
       */
      slot(voffset) {
        if (this.vtable !== null)
          this.vtable[voffset] = this.offset();
      }
      /**
       * @returns Offset relative to the end of the buffer.
       */
      offset() {
        return this.bb.capacity() - this.space;
      }
      /**
       * Doubles the size of the backing ByteBuffer and copies the old data towards
       * the end of the new buffer (since we build the buffer backwards).
       *
       * @param bb The current buffer with the existing data
       * @returns A new byte buffer with the old data copied
       * to it. The data is located at the end of the buffer.
       *
       * uint8Array.set() formally takes {Array<number>|ArrayBufferView}, so to pass
       * it a uint8Array we need to suppress the type check:
       * @suppress {checkTypes}
       */
      static growByteBuffer(bb) {
        const old_buf_size = bb.capacity();
        if (old_buf_size & 3221225472) {
          throw new Error("FlatBuffers: cannot grow buffer beyond 2 gigabytes.");
        }
        const new_buf_size = old_buf_size << 1;
        const nbb = byte_buffer_js_1.ByteBuffer.allocate(new_buf_size);
        nbb.setPosition(new_buf_size - old_buf_size);
        nbb.bytes().set(bb.bytes(), new_buf_size - old_buf_size);
        return nbb;
      }
      /**
       * Adds on offset, relative to where it will be written.
       *
       * @param offset The offset to add.
       */
      addOffset(offset) {
        this.prep(constants_js_1.SIZEOF_INT, 0);
        this.writeInt32(this.offset() - offset + constants_js_1.SIZEOF_INT);
      }
      /**
       * Start encoding a new object in the buffer.  Users will not usually need to
       * call this directly. The FlatBuffers compiler will generate helper methods
       * that call this method internally.
       */
      startObject(numfields) {
        this.notNested();
        if (this.vtable == null) {
          this.vtable = [];
        }
        this.vtable_in_use = numfields;
        for (let i = 0; i < numfields; i++) {
          this.vtable[i] = 0;
        }
        this.isNested = true;
        this.object_start = this.offset();
      }
      /**
       * Finish off writing the object that is under construction.
       *
       * @returns The offset to the object inside `dataBuffer`
       */
      endObject() {
        if (this.vtable == null || !this.isNested) {
          throw new Error("FlatBuffers: endObject called without startObject");
        }
        this.addInt32(0);
        const vtableloc = this.offset();
        let i = this.vtable_in_use - 1;
        for (; i >= 0 && this.vtable[i] == 0; i--) {
        }
        const trimmed_size = i + 1;
        for (; i >= 0; i--) {
          this.addInt16(this.vtable[i] != 0 ? vtableloc - this.vtable[i] : 0);
        }
        const standard_fields = 2;
        this.addInt16(vtableloc - this.object_start);
        const len = (trimmed_size + standard_fields) * constants_js_1.SIZEOF_SHORT;
        this.addInt16(len);
        let existing_vtable = 0;
        const vt1 = this.space;
        outer_loop: for (i = 0; i < this.vtables.length; i++) {
          const vt2 = this.bb.capacity() - this.vtables[i];
          if (len == this.bb.readInt16(vt2)) {
            for (let j = constants_js_1.SIZEOF_SHORT; j < len; j += constants_js_1.SIZEOF_SHORT) {
              if (this.bb.readInt16(vt1 + j) != this.bb.readInt16(vt2 + j)) {
                continue outer_loop;
              }
            }
            existing_vtable = this.vtables[i];
            break;
          }
        }
        if (existing_vtable) {
          this.space = this.bb.capacity() - vtableloc;
          this.bb.writeInt32(this.space, existing_vtable - vtableloc);
        } else {
          this.vtables.push(this.offset());
          this.bb.writeInt32(this.bb.capacity() - vtableloc, this.offset() - vtableloc);
        }
        this.isNested = false;
        return vtableloc;
      }
      /**
       * Finalize a buffer, poiting to the given `root_table`.
       */
      finish(root_table, opt_file_identifier, opt_size_prefix) {
        const size_prefix = opt_size_prefix ? constants_js_1.SIZE_PREFIX_LENGTH : 0;
        if (opt_file_identifier) {
          const file_identifier = opt_file_identifier;
          this.prep(this.minalign, constants_js_1.SIZEOF_INT + constants_js_1.FILE_IDENTIFIER_LENGTH + size_prefix);
          if (file_identifier.length != constants_js_1.FILE_IDENTIFIER_LENGTH) {
            throw new TypeError("FlatBuffers: file identifier must be length " + constants_js_1.FILE_IDENTIFIER_LENGTH);
          }
          for (let i = constants_js_1.FILE_IDENTIFIER_LENGTH - 1; i >= 0; i--) {
            this.writeInt8(file_identifier.charCodeAt(i));
          }
        }
        this.prep(this.minalign, constants_js_1.SIZEOF_INT + size_prefix);
        this.addOffset(root_table);
        if (size_prefix) {
          this.addInt32(this.bb.capacity() - this.space);
        }
        this.bb.setPosition(this.space);
      }
      /**
       * Finalize a size prefixed buffer, pointing to the given `root_table`.
       */
      finishSizePrefixed(root_table, opt_file_identifier) {
        this.finish(root_table, opt_file_identifier, true);
      }
      /**
       * This checks a required field has been set in a given table that has
       * just been constructed.
       */
      requiredField(table, field) {
        const table_start = this.bb.capacity() - table;
        const vtable_start = table_start - this.bb.readInt32(table_start);
        const ok = field < this.bb.readInt16(vtable_start) && this.bb.readInt16(vtable_start + field) != 0;
        if (!ok) {
          throw new TypeError("FlatBuffers: field " + field + " must be set");
        }
      }
      /**
       * Start a new array/vector of objects.  Users usually will not call
       * this directly. The FlatBuffers compiler will create a start/end
       * method for vector types in generated code.
       *
       * @param elem_size The size of each element in the array
       * @param num_elems The number of elements in the array
       * @param alignment The alignment of the array
       */
      startVector(elem_size, num_elems, alignment) {
        this.notNested();
        this.vector_num_elems = num_elems;
        this.prep(constants_js_1.SIZEOF_INT, elem_size * num_elems);
        this.prep(alignment, elem_size * num_elems);
      }
      /**
       * Finish off the creation of an array and all its elements. The array must be
       * created with `startVector`.
       *
       * @returns The offset at which the newly created array
       * starts.
       */
      endVector() {
        this.writeInt32(this.vector_num_elems);
        return this.offset();
      }
      /**
       * Encode the string `s` in the buffer using UTF-8. If the string passed has
       * already been seen, we return the offset of the already written string
       *
       * @param s The string to encode
       * @return The offset in the buffer where the encoded string starts
       */
      createSharedString(s) {
        if (!s) {
          return 0;
        }
        if (!this.string_maps) {
          this.string_maps = /* @__PURE__ */ new Map();
        }
        if (this.string_maps.has(s)) {
          return this.string_maps.get(s);
        }
        const offset = this.createString(s);
        this.string_maps.set(s, offset);
        return offset;
      }
      /**
       * Encode the string `s` in the buffer using UTF-8. If a Uint8Array is passed
       * instead of a string, it is assumed to contain valid UTF-8 encoded data.
       *
       * @param s The string to encode
       * @return The offset in the buffer where the encoded string starts
       */
      createString(s) {
        if (s === null || s === void 0) {
          return 0;
        }
        let utf8;
        if (s instanceof Uint8Array) {
          utf8 = s;
        } else {
          utf8 = this.text_encoder.encode(s);
        }
        this.addInt8(0);
        this.startVector(1, utf8.length, 1);
        this.bb.setPosition(this.space -= utf8.length);
        this.bb.bytes().set(utf8, this.space);
        return this.endVector();
      }
      /**
       * Create a byte vector.
       *
       * @param v The bytes to add
       * @returns The offset in the buffer where the byte vector starts
       */
      createByteVector(v) {
        if (v === null || v === void 0) {
          return 0;
        }
        this.startVector(1, v.length, 1);
        this.bb.setPosition(this.space -= v.length);
        this.bb.bytes().set(v, this.space);
        return this.endVector();
      }
      /**
       * A helper function to pack an object
       *
       * @returns offset of obj
       */
      createObjectOffset(obj) {
        if (obj === null) {
          return 0;
        }
        if (typeof obj === "string") {
          return this.createString(obj);
        } else {
          return obj.pack(this);
        }
      }
      /**
       * A helper function to pack a list of object
       *
       * @returns list of offsets of each non null object
       */
      createObjectOffsetList(list) {
        const ret = [];
        for (let i = 0; i < list.length; ++i) {
          const val = list[i];
          if (val !== null) {
            ret.push(this.createObjectOffset(val));
          } else {
            throw new TypeError("FlatBuffers: Argument for createObjectOffsetList cannot contain null.");
          }
        }
        return ret;
      }
      createStructOffsetList(list, startFunc) {
        startFunc(this, list.length);
        this.createObjectOffsetList(list.slice().reverse());
        return this.endVector();
      }
    };
    exports.Builder = Builder;
  }
});

// ../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/flatbuffers.js
var require_flatbuffers = __commonJS({
  "../../../../../../tmp/revault-jsdeps/node_modules/flatbuffers/js/flatbuffers.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Encoding = exports.ByteBuffer = exports.Builder = exports.isLittleEndian = exports.int32 = exports.float64 = exports.float32 = exports.SIZE_PREFIX_LENGTH = exports.SIZEOF_SHORT = exports.SIZEOF_INT = exports.FILE_IDENTIFIER_LENGTH = void 0;
    var constants_js_1 = require_constants();
    Object.defineProperty(exports, "FILE_IDENTIFIER_LENGTH", { enumerable: true, get: function() {
      return constants_js_1.FILE_IDENTIFIER_LENGTH;
    } });
    Object.defineProperty(exports, "SIZEOF_INT", { enumerable: true, get: function() {
      return constants_js_1.SIZEOF_INT;
    } });
    Object.defineProperty(exports, "SIZEOF_SHORT", { enumerable: true, get: function() {
      return constants_js_1.SIZEOF_SHORT;
    } });
    Object.defineProperty(exports, "SIZE_PREFIX_LENGTH", { enumerable: true, get: function() {
      return constants_js_1.SIZE_PREFIX_LENGTH;
    } });
    var utils_js_1 = require_utils();
    Object.defineProperty(exports, "float32", { enumerable: true, get: function() {
      return utils_js_1.float32;
    } });
    Object.defineProperty(exports, "float64", { enumerable: true, get: function() {
      return utils_js_1.float64;
    } });
    Object.defineProperty(exports, "int32", { enumerable: true, get: function() {
      return utils_js_1.int32;
    } });
    Object.defineProperty(exports, "isLittleEndian", { enumerable: true, get: function() {
      return utils_js_1.isLittleEndian;
    } });
    var builder_js_1 = require_builder();
    Object.defineProperty(exports, "Builder", { enumerable: true, get: function() {
      return builder_js_1.Builder;
    } });
    var byte_buffer_js_1 = require_byte_buffer();
    Object.defineProperty(exports, "ByteBuffer", { enumerable: true, get: function() {
      return byte_buffer_js_1.ByteBuffer;
    } });
    var encoding_js_1 = require_encoding();
    Object.defineProperty(exports, "Encoding", { enumerable: true, get: function() {
      return encoding_js_1.Encoding;
    } });
  }
});

// generated/flatbuffers/revault/internal/access-slot-label.ts
var flatbuffers = __toESM(require_flatbuffers(), 1);
var AccessSlotLabel = class _AccessSlotLabel {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsAccessSlotLabel(bb, obj) {
    return (obj || new _AccessSlotLabel()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsAccessSlotLabel(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
    return (obj || new _AccessSlotLabel()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  lockboxId(index) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  lockboxIdLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  lockboxIdArray() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  slotId() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  updatedAtUnixMs() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  static startAccessSlotLabel(builder) {
    builder.startObject(4);
  }
  static addLockboxId(builder, lockboxIdOffset) {
    builder.addFieldOffset(0, lockboxIdOffset, 0);
  }
  static createLockboxIdVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startLockboxIdVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addSlotId(builder, slotId) {
    builder.addFieldInt64(1, slotId, BigInt("0"));
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(2, nameOffset, 0);
  }
  static addUpdatedAtUnixMs(builder, updatedAtUnixMs) {
    builder.addFieldInt64(3, updatedAtUnixMs, BigInt("0"));
  }
  static endAccessSlotLabel(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createAccessSlotLabel(builder, lockboxIdOffset, slotId, nameOffset, updatedAtUnixMs) {
    _AccessSlotLabel.startAccessSlotLabel(builder);
    _AccessSlotLabel.addLockboxId(builder, lockboxIdOffset);
    _AccessSlotLabel.addSlotId(builder, slotId);
    _AccessSlotLabel.addName(builder, nameOffset);
    _AccessSlotLabel.addUpdatedAtUnixMs(builder, updatedAtUnixMs);
    return _AccessSlotLabel.endAccessSlotLabel(builder);
  }
  unpack() {
    return new AccessSlotLabelT(
      this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength()),
      this.slotId(),
      this.name(),
      this.updatedAtUnixMs()
    );
  }
  unpackTo(_o) {
    _o.lockboxId = this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength());
    _o.slotId = this.slotId();
    _o.name = this.name();
    _o.updatedAtUnixMs = this.updatedAtUnixMs();
  }
};
var AccessSlotLabelT = class {
  constructor(lockboxId = [], slotId = BigInt("0"), name = null, updatedAtUnixMs = BigInt("0")) {
    this.lockboxId = lockboxId;
    this.slotId = slotId;
    this.name = name;
    this.updatedAtUnixMs = updatedAtUnixMs;
  }
  pack(builder) {
    const lockboxId = AccessSlotLabel.createLockboxIdVector(builder, this.lockboxId);
    const name = this.name !== null ? builder.createString(this.name) : 0;
    return AccessSlotLabel.createAccessSlotLabel(
      builder,
      lockboxId,
      this.slotId,
      name,
      this.updatedAtUnixMs
    );
  }
};

// generated/flatbuffers/revault/internal/access-slot-label-list.ts
var flatbuffers2 = __toESM(require_flatbuffers(), 1);
var AccessSlotLabelList = class _AccessSlotLabelList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsAccessSlotLabelList(bb, obj) {
    return (obj || new _AccessSlotLabelList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsAccessSlotLabelList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers2.SIZE_PREFIX_LENGTH);
    return (obj || new _AccessSlotLabelList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new AccessSlotLabel()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startAccessSlotLabelList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endAccessSlotLabelList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createAccessSlotLabelList(builder, valuesOffset) {
    _AccessSlotLabelList.startAccessSlotLabelList(builder);
    _AccessSlotLabelList.addValues(builder, valuesOffset);
    return _AccessSlotLabelList.endAccessSlotLabelList(builder);
  }
  unpack() {
    return new AccessSlotLabelListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var AccessSlotLabelListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = AccessSlotLabelList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return AccessSlotLabelList.createAccessSlotLabelList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/agent-entry.ts
var flatbuffers3 = __toESM(require_flatbuffers(), 1);
var AgentEntry = class _AgentEntry {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsAgentEntry(bb, obj) {
    return (obj || new _AgentEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsAgentEntry(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers3.SIZE_PREFIX_LENGTH);
    return (obj || new _AgentEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  path(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startAgentEntry(builder) {
    builder.startObject(2);
  }
  static addId(builder, idOffset) {
    builder.addFieldOffset(0, idOffset, 0);
  }
  static addPath(builder, pathOffset) {
    builder.addFieldOffset(1, pathOffset, 0);
  }
  static endAgentEntry(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createAgentEntry(builder, idOffset, pathOffset) {
    _AgentEntry.startAgentEntry(builder);
    _AgentEntry.addId(builder, idOffset);
    _AgentEntry.addPath(builder, pathOffset);
    return _AgentEntry.endAgentEntry(builder);
  }
  unpack() {
    return new AgentEntryT(
      this.id(),
      this.path()
    );
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.path = this.path();
  }
};
var AgentEntryT = class {
  constructor(id = null, path = null) {
    this.id = id;
    this.path = path;
  }
  pack(builder) {
    const id = this.id !== null ? builder.createString(this.id) : 0;
    const path = this.path !== null ? builder.createString(this.path) : 0;
    return AgentEntry.createAgentEntry(
      builder,
      id,
      path
    );
  }
};

// generated/flatbuffers/revault/internal/agent-entry-list.ts
var flatbuffers4 = __toESM(require_flatbuffers(), 1);
var AgentEntryList = class _AgentEntryList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsAgentEntryList(bb, obj) {
    return (obj || new _AgentEntryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsAgentEntryList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers4.SIZE_PREFIX_LENGTH);
    return (obj || new _AgentEntryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new AgentEntry()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startAgentEntryList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endAgentEntryList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createAgentEntryList(builder, valuesOffset) {
    _AgentEntryList.startAgentEntryList(builder);
    _AgentEntryList.addValues(builder, valuesOffset);
    return _AgentEntryList.endAgentEntryList(builder);
  }
  unpack() {
    return new AgentEntryListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var AgentEntryListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = AgentEntryList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return AgentEntryList.createAgentEntryList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/cache-stats.ts
var flatbuffers5 = __toESM(require_flatbuffers(), 1);
var CacheStats = class _CacheStats {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsCacheStats(bb, obj) {
    return (obj || new _CacheStats()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsCacheStats(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers5.SIZE_PREFIX_LENGTH);
    return (obj || new _CacheStats()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  limitBytes() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  usedBytes() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  entries() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  hits() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  misses() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  static startCacheStats(builder) {
    builder.startObject(5);
  }
  static addLimitBytes(builder, limitBytes) {
    builder.addFieldInt64(0, limitBytes, BigInt("0"));
  }
  static addUsedBytes(builder, usedBytes) {
    builder.addFieldInt64(1, usedBytes, BigInt("0"));
  }
  static addEntries(builder, entries) {
    builder.addFieldInt64(2, entries, BigInt("0"));
  }
  static addHits(builder, hits) {
    builder.addFieldInt64(3, hits, BigInt("0"));
  }
  static addMisses(builder, misses) {
    builder.addFieldInt64(4, misses, BigInt("0"));
  }
  static endCacheStats(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createCacheStats(builder, limitBytes, usedBytes, entries, hits, misses) {
    _CacheStats.startCacheStats(builder);
    _CacheStats.addLimitBytes(builder, limitBytes);
    _CacheStats.addUsedBytes(builder, usedBytes);
    _CacheStats.addEntries(builder, entries);
    _CacheStats.addHits(builder, hits);
    _CacheStats.addMisses(builder, misses);
    return _CacheStats.endCacheStats(builder);
  }
  unpack() {
    return new CacheStatsT(
      this.limitBytes(),
      this.usedBytes(),
      this.entries(),
      this.hits(),
      this.misses()
    );
  }
  unpackTo(_o) {
    _o.limitBytes = this.limitBytes();
    _o.usedBytes = this.usedBytes();
    _o.entries = this.entries();
    _o.hits = this.hits();
    _o.misses = this.misses();
  }
};
var CacheStatsT = class {
  constructor(limitBytes = BigInt("0"), usedBytes = BigInt("0"), entries = BigInt("0"), hits = BigInt("0"), misses = BigInt("0")) {
    this.limitBytes = limitBytes;
    this.usedBytes = usedBytes;
    this.entries = entries;
    this.hits = hits;
    this.misses = misses;
  }
  pack(builder) {
    return CacheStats.createCacheStats(
      builder,
      this.limitBytes,
      this.usedBytes,
      this.entries,
      this.hits,
      this.misses
    );
  }
};

// generated/flatbuffers/revault/internal/contact.ts
var flatbuffers6 = __toESM(require_flatbuffers(), 1);
var Contact = class _Contact {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsContact(bb, obj) {
    return (obj || new _Contact()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsContact(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers6.SIZE_PREFIX_LENGTH);
    return (obj || new _Contact()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  key(index) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  keyLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  keyArray() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  static startContact(builder) {
    builder.startObject(2);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addKey(builder, keyOffset) {
    builder.addFieldOffset(1, keyOffset, 0);
  }
  static createKeyVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startKeyVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static endContact(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createContact(builder, nameOffset, keyOffset) {
    _Contact.startContact(builder);
    _Contact.addName(builder, nameOffset);
    _Contact.addKey(builder, keyOffset);
    return _Contact.endContact(builder);
  }
  unpack() {
    return new ContactT(
      this.name(),
      this.bb.createScalarList(this.key.bind(this), this.keyLength())
    );
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.key = this.bb.createScalarList(this.key.bind(this), this.keyLength());
  }
};
var ContactT = class {
  constructor(name = null, key = []) {
    this.name = name;
    this.key = key;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const key = Contact.createKeyVector(builder, this.key);
    return Contact.createContact(
      builder,
      name,
      key
    );
  }
};

// generated/flatbuffers/revault/internal/contact-list.ts
var flatbuffers7 = __toESM(require_flatbuffers(), 1);
var ContactList = class _ContactList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsContactList(bb, obj) {
    return (obj || new _ContactList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsContactList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers7.SIZE_PREFIX_LENGTH);
    return (obj || new _ContactList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new Contact()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startContactList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endContactList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createContactList(builder, valuesOffset) {
    _ContactList.startContactList(builder);
    _ContactList.addValues(builder, valuesOffset);
    return _ContactList.endContactList(builder);
  }
  unpack() {
    return new ContactListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var ContactListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = ContactList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return ContactList.createContactList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/error-details.ts
var flatbuffers8 = __toESM(require_flatbuffers(), 1);
var ErrorDetails = class _ErrorDetails {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsErrorDetails(bb, obj) {
    return (obj || new _ErrorDetails()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsErrorDetails(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers8.SIZE_PREFIX_LENGTH);
    return (obj || new _ErrorDetails()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  category(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  artifactKind(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  foundVersion() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  supportedVersion() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  message(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  guidance(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startErrorDetails(builder) {
    builder.startObject(6);
  }
  static addCategory(builder, categoryOffset) {
    builder.addFieldOffset(0, categoryOffset, 0);
  }
  static addArtifactKind(builder, artifactKindOffset) {
    builder.addFieldOffset(1, artifactKindOffset, 0);
  }
  static addFoundVersion(builder, foundVersion) {
    builder.addFieldInt32(2, foundVersion, 0);
  }
  static addSupportedVersion(builder, supportedVersion) {
    builder.addFieldInt32(3, supportedVersion, 0);
  }
  static addMessage(builder, messageOffset) {
    builder.addFieldOffset(4, messageOffset, 0);
  }
  static addGuidance(builder, guidanceOffset) {
    builder.addFieldOffset(5, guidanceOffset, 0);
  }
  static endErrorDetails(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createErrorDetails(builder, categoryOffset, artifactKindOffset, foundVersion, supportedVersion, messageOffset, guidanceOffset) {
    _ErrorDetails.startErrorDetails(builder);
    _ErrorDetails.addCategory(builder, categoryOffset);
    _ErrorDetails.addArtifactKind(builder, artifactKindOffset);
    _ErrorDetails.addFoundVersion(builder, foundVersion);
    _ErrorDetails.addSupportedVersion(builder, supportedVersion);
    _ErrorDetails.addMessage(builder, messageOffset);
    _ErrorDetails.addGuidance(builder, guidanceOffset);
    return _ErrorDetails.endErrorDetails(builder);
  }
  unpack() {
    return new ErrorDetailsT(
      this.category(),
      this.artifactKind(),
      this.foundVersion(),
      this.supportedVersion(),
      this.message(),
      this.guidance()
    );
  }
  unpackTo(_o) {
    _o.category = this.category();
    _o.artifactKind = this.artifactKind();
    _o.foundVersion = this.foundVersion();
    _o.supportedVersion = this.supportedVersion();
    _o.message = this.message();
    _o.guidance = this.guidance();
  }
};
var ErrorDetailsT = class {
  constructor(category = null, artifactKind = null, foundVersion = 0, supportedVersion = 0, message = null, guidance = null) {
    this.category = category;
    this.artifactKind = artifactKind;
    this.foundVersion = foundVersion;
    this.supportedVersion = supportedVersion;
    this.message = message;
    this.guidance = guidance;
  }
  pack(builder) {
    const category = this.category !== null ? builder.createString(this.category) : 0;
    const artifactKind = this.artifactKind !== null ? builder.createString(this.artifactKind) : 0;
    const message = this.message !== null ? builder.createString(this.message) : 0;
    const guidance = this.guidance !== null ? builder.createString(this.guidance) : 0;
    return ErrorDetails.createErrorDetails(
      builder,
      category,
      artifactKind,
      this.foundVersion,
      this.supportedVersion,
      message,
      guidance
    );
  }
};

// generated/flatbuffers/revault/internal/file-inspection.ts
var flatbuffers10 = __toESM(require_flatbuffers(), 1);

// generated/flatbuffers/revault/internal/key-slot.ts
var flatbuffers9 = __toESM(require_flatbuffers(), 1);
var KeySlot = class _KeySlot {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsKeySlot(bb, obj) {
    return (obj || new _KeySlot()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsKeySlot(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers9.SIZE_PREFIX_LENGTH);
    return (obj || new _KeySlot()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  protection(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  algorithm(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startKeySlot(builder) {
    builder.startObject(3);
  }
  static addId(builder, id) {
    builder.addFieldInt64(0, id, BigInt("0"));
  }
  static addProtection(builder, protectionOffset) {
    builder.addFieldOffset(1, protectionOffset, 0);
  }
  static addAlgorithm(builder, algorithmOffset) {
    builder.addFieldOffset(2, algorithmOffset, 0);
  }
  static endKeySlot(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createKeySlot(builder, id, protectionOffset, algorithmOffset) {
    _KeySlot.startKeySlot(builder);
    _KeySlot.addId(builder, id);
    _KeySlot.addProtection(builder, protectionOffset);
    _KeySlot.addAlgorithm(builder, algorithmOffset);
    return _KeySlot.endKeySlot(builder);
  }
  unpack() {
    return new KeySlotT(
      this.id(),
      this.protection(),
      this.algorithm()
    );
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.protection = this.protection();
    _o.algorithm = this.algorithm();
  }
};
var KeySlotT = class {
  constructor(id = BigInt("0"), protection = null, algorithm = null) {
    this.id = id;
    this.protection = protection;
    this.algorithm = algorithm;
  }
  pack(builder) {
    const protection = this.protection !== null ? builder.createString(this.protection) : 0;
    const algorithm = this.algorithm !== null ? builder.createString(this.algorithm) : 0;
    return KeySlot.createKeySlot(
      builder,
      this.id,
      protection,
      algorithm
    );
  }
};

// generated/flatbuffers/revault/internal/file-inspection.ts
var FileInspection = class _FileInspection {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFileInspection(bb, obj) {
    return (obj || new _FileInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFileInspection(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers10.SIZE_PREFIX_LENGTH);
    return (obj || new _FileInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  lockboxId(index) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  lockboxIdLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  lockboxIdArray() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  headerReadable() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  keyDirectoryGeneration() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  keyDirectoryCopyCount() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  ownerSigned() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  keySlots(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new KeySlot()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  keySlotsLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFileInspection(builder) {
    builder.startObject(6);
  }
  static addLockboxId(builder, lockboxIdOffset) {
    builder.addFieldOffset(0, lockboxIdOffset, 0);
  }
  static createLockboxIdVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startLockboxIdVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addHeaderReadable(builder, headerReadable) {
    builder.addFieldInt8(1, +headerReadable, 0);
  }
  static addKeyDirectoryGeneration(builder, keyDirectoryGeneration) {
    builder.addFieldInt64(2, keyDirectoryGeneration, BigInt("0"));
  }
  static addKeyDirectoryCopyCount(builder, keyDirectoryCopyCount) {
    builder.addFieldInt64(3, keyDirectoryCopyCount, BigInt("0"));
  }
  static addOwnerSigned(builder, ownerSigned) {
    builder.addFieldInt8(4, +ownerSigned, 0);
  }
  static addKeySlots(builder, keySlotsOffset) {
    builder.addFieldOffset(5, keySlotsOffset, 0);
  }
  static createKeySlotsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startKeySlotsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFileInspection(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFileInspection(builder, lockboxIdOffset, headerReadable, keyDirectoryGeneration, keyDirectoryCopyCount, ownerSigned, keySlotsOffset) {
    _FileInspection.startFileInspection(builder);
    _FileInspection.addLockboxId(builder, lockboxIdOffset);
    _FileInspection.addHeaderReadable(builder, headerReadable);
    _FileInspection.addKeyDirectoryGeneration(builder, keyDirectoryGeneration);
    _FileInspection.addKeyDirectoryCopyCount(builder, keyDirectoryCopyCount);
    _FileInspection.addOwnerSigned(builder, ownerSigned);
    _FileInspection.addKeySlots(builder, keySlotsOffset);
    return _FileInspection.endFileInspection(builder);
  }
  unpack() {
    return new FileInspectionT(
      this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength()),
      this.headerReadable(),
      this.keyDirectoryGeneration(),
      this.keyDirectoryCopyCount(),
      this.ownerSigned(),
      this.bb.createObjList(this.keySlots.bind(this), this.keySlotsLength())
    );
  }
  unpackTo(_o) {
    _o.lockboxId = this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength());
    _o.headerReadable = this.headerReadable();
    _o.keyDirectoryGeneration = this.keyDirectoryGeneration();
    _o.keyDirectoryCopyCount = this.keyDirectoryCopyCount();
    _o.ownerSigned = this.ownerSigned();
    _o.keySlots = this.bb.createObjList(this.keySlots.bind(this), this.keySlotsLength());
  }
};
var FileInspectionT = class {
  constructor(lockboxId = [], headerReadable = false, keyDirectoryGeneration = BigInt("0"), keyDirectoryCopyCount = BigInt("0"), ownerSigned = false, keySlots = []) {
    this.lockboxId = lockboxId;
    this.headerReadable = headerReadable;
    this.keyDirectoryGeneration = keyDirectoryGeneration;
    this.keyDirectoryCopyCount = keyDirectoryCopyCount;
    this.ownerSigned = ownerSigned;
    this.keySlots = keySlots;
  }
  pack(builder) {
    const lockboxId = FileInspection.createLockboxIdVector(builder, this.lockboxId);
    const keySlots = FileInspection.createKeySlotsVector(builder, builder.createObjectOffsetList(this.keySlots));
    return FileInspection.createFileInspection(
      builder,
      lockboxId,
      this.headerReadable,
      this.keyDirectoryGeneration,
      this.keyDirectoryCopyCount,
      this.ownerSigned,
      keySlots
    );
  }
};

// generated/flatbuffers/revault/internal/form-definition.ts
var flatbuffers12 = __toESM(require_flatbuffers(), 1);

// generated/flatbuffers/revault/internal/form-field.ts
var flatbuffers11 = __toESM(require_flatbuffers(), 1);
var FormField = class _FormField {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormField(bb, obj) {
    return (obj || new _FormField()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormField(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers11.SIZE_PREFIX_LENGTH);
    return (obj || new _FormField()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  label(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  kind(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  required() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  static startFormField(builder) {
    builder.startObject(4);
  }
  static addId(builder, idOffset) {
    builder.addFieldOffset(0, idOffset, 0);
  }
  static addLabel(builder, labelOffset) {
    builder.addFieldOffset(1, labelOffset, 0);
  }
  static addKind(builder, kindOffset) {
    builder.addFieldOffset(2, kindOffset, 0);
  }
  static addRequired(builder, required) {
    builder.addFieldInt8(3, +required, 0);
  }
  static endFormField(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormField(builder, idOffset, labelOffset, kindOffset, required) {
    _FormField.startFormField(builder);
    _FormField.addId(builder, idOffset);
    _FormField.addLabel(builder, labelOffset);
    _FormField.addKind(builder, kindOffset);
    _FormField.addRequired(builder, required);
    return _FormField.endFormField(builder);
  }
  unpack() {
    return new FormFieldT(
      this.id(),
      this.label(),
      this.kind(),
      this.required()
    );
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.label = this.label();
    _o.kind = this.kind();
    _o.required = this.required();
  }
};
var FormFieldT = class {
  constructor(id = null, label = null, kind = null, required = false) {
    this.id = id;
    this.label = label;
    this.kind = kind;
    this.required = required;
  }
  pack(builder) {
    const id = this.id !== null ? builder.createString(this.id) : 0;
    const label = this.label !== null ? builder.createString(this.label) : 0;
    const kind = this.kind !== null ? builder.createString(this.kind) : 0;
    return FormField.createFormField(
      builder,
      id,
      label,
      kind,
      this.required
    );
  }
};

// generated/flatbuffers/revault/internal/form-definition.ts
var FormDefinition = class _FormDefinition {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormDefinition(bb, obj) {
    return (obj || new _FormDefinition()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormDefinition(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers12.SIZE_PREFIX_LENGTH);
    return (obj || new _FormDefinition()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  typeId(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  alias(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  revision() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  description(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  fields(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new FormField()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  fieldsLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFormDefinition(builder) {
    builder.startObject(6);
  }
  static addTypeId(builder, typeIdOffset) {
    builder.addFieldOffset(0, typeIdOffset, 0);
  }
  static addAlias(builder, aliasOffset) {
    builder.addFieldOffset(1, aliasOffset, 0);
  }
  static addRevision(builder, revision) {
    builder.addFieldInt32(2, revision, 0);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(3, nameOffset, 0);
  }
  static addDescription(builder, descriptionOffset) {
    builder.addFieldOffset(4, descriptionOffset, 0);
  }
  static addFields(builder, fieldsOffset) {
    builder.addFieldOffset(5, fieldsOffset, 0);
  }
  static createFieldsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startFieldsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFormDefinition(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormDefinition(builder, typeIdOffset, aliasOffset, revision, nameOffset, descriptionOffset, fieldsOffset) {
    _FormDefinition.startFormDefinition(builder);
    _FormDefinition.addTypeId(builder, typeIdOffset);
    _FormDefinition.addAlias(builder, aliasOffset);
    _FormDefinition.addRevision(builder, revision);
    _FormDefinition.addName(builder, nameOffset);
    _FormDefinition.addDescription(builder, descriptionOffset);
    _FormDefinition.addFields(builder, fieldsOffset);
    return _FormDefinition.endFormDefinition(builder);
  }
  unpack() {
    return new FormDefinitionT(
      this.typeId(),
      this.alias(),
      this.revision(),
      this.name(),
      this.description(),
      this.bb.createObjList(this.fields.bind(this), this.fieldsLength())
    );
  }
  unpackTo(_o) {
    _o.typeId = this.typeId();
    _o.alias = this.alias();
    _o.revision = this.revision();
    _o.name = this.name();
    _o.description = this.description();
    _o.fields = this.bb.createObjList(this.fields.bind(this), this.fieldsLength());
  }
};
var FormDefinitionT = class {
  constructor(typeId = null, alias = null, revision = 0, name = null, description = null, fields = []) {
    this.typeId = typeId;
    this.alias = alias;
    this.revision = revision;
    this.name = name;
    this.description = description;
    this.fields = fields;
  }
  pack(builder) {
    const typeId = this.typeId !== null ? builder.createString(this.typeId) : 0;
    const alias = this.alias !== null ? builder.createString(this.alias) : 0;
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const description = this.description !== null ? builder.createString(this.description) : 0;
    const fields = FormDefinition.createFieldsVector(builder, builder.createObjectOffsetList(this.fields));
    return FormDefinition.createFormDefinition(
      builder,
      typeId,
      alias,
      this.revision,
      name,
      description,
      fields
    );
  }
};

// generated/flatbuffers/revault/internal/form-definition-list.ts
var flatbuffers13 = __toESM(require_flatbuffers(), 1);
var FormDefinitionList = class _FormDefinitionList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormDefinitionList(bb, obj) {
    return (obj || new _FormDefinitionList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormDefinitionList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers13.SIZE_PREFIX_LENGTH);
    return (obj || new _FormDefinitionList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new FormDefinition()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFormDefinitionList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFormDefinitionList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormDefinitionList(builder, valuesOffset) {
    _FormDefinitionList.startFormDefinitionList(builder);
    _FormDefinitionList.addValues(builder, valuesOffset);
    return _FormDefinitionList.endFormDefinitionList(builder);
  }
  unpack() {
    return new FormDefinitionListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var FormDefinitionListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = FormDefinitionList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return FormDefinitionList.createFormDefinitionList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/form-field-list.ts
var flatbuffers14 = __toESM(require_flatbuffers(), 1);
var FormFieldList = class _FormFieldList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormFieldList(bb, obj) {
    return (obj || new _FormFieldList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormFieldList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers14.SIZE_PREFIX_LENGTH);
    return (obj || new _FormFieldList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new FormField()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFormFieldList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFormFieldList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormFieldList(builder, valuesOffset) {
    _FormFieldList.startFormFieldList(builder);
    _FormFieldList.addValues(builder, valuesOffset);
    return _FormFieldList.endFormFieldList(builder);
  }
  unpack() {
    return new FormFieldListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var FormFieldListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = FormFieldList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return FormFieldList.createFormFieldList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/form-record.ts
var flatbuffers16 = __toESM(require_flatbuffers(), 1);

// generated/flatbuffers/revault/internal/form-value.ts
var flatbuffers15 = __toESM(require_flatbuffers(), 1);
var FormValue = class _FormValue {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormValue(bb, obj) {
    return (obj || new _FormValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormValue(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers15.SIZE_PREFIX_LENGTH);
    return (obj || new _FormValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  fieldId(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  label(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  kind(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  value(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  secret() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  static startFormValue(builder) {
    builder.startObject(5);
  }
  static addFieldId(builder, fieldIdOffset) {
    builder.addFieldOffset(0, fieldIdOffset, 0);
  }
  static addLabel(builder, labelOffset) {
    builder.addFieldOffset(1, labelOffset, 0);
  }
  static addKind(builder, kindOffset) {
    builder.addFieldOffset(2, kindOffset, 0);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(3, valueOffset, 0);
  }
  static addSecret(builder, secret) {
    builder.addFieldInt8(4, +secret, 0);
  }
  static endFormValue(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormValue(builder, fieldIdOffset, labelOffset, kindOffset, valueOffset, secret) {
    _FormValue.startFormValue(builder);
    _FormValue.addFieldId(builder, fieldIdOffset);
    _FormValue.addLabel(builder, labelOffset);
    _FormValue.addKind(builder, kindOffset);
    _FormValue.addValue(builder, valueOffset);
    _FormValue.addSecret(builder, secret);
    return _FormValue.endFormValue(builder);
  }
  unpack() {
    return new FormValueT(
      this.fieldId(),
      this.label(),
      this.kind(),
      this.value(),
      this.secret()
    );
  }
  unpackTo(_o) {
    _o.fieldId = this.fieldId();
    _o.label = this.label();
    _o.kind = this.kind();
    _o.value = this.value();
    _o.secret = this.secret();
  }
};
var FormValueT = class {
  constructor(fieldId = null, label = null, kind = null, value = null, secret = false) {
    this.fieldId = fieldId;
    this.label = label;
    this.kind = kind;
    this.value = value;
    this.secret = secret;
  }
  pack(builder) {
    const fieldId = this.fieldId !== null ? builder.createString(this.fieldId) : 0;
    const label = this.label !== null ? builder.createString(this.label) : 0;
    const kind = this.kind !== null ? builder.createString(this.kind) : 0;
    const value = this.value !== null ? builder.createString(this.value) : 0;
    return FormValue.createFormValue(
      builder,
      fieldId,
      label,
      kind,
      value,
      this.secret
    );
  }
};

// generated/flatbuffers/revault/internal/form-record.ts
var FormRecord = class _FormRecord {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormRecord(bb, obj) {
    return (obj || new _FormRecord()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormRecord(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers16.SIZE_PREFIX_LENGTH);
    return (obj || new _FormRecord()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  path(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  typeId(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  definitionAlias(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  definitionRevision() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new FormValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFormRecord(builder) {
    builder.startObject(6);
  }
  static addPath(builder, pathOffset) {
    builder.addFieldOffset(0, pathOffset, 0);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(1, nameOffset, 0);
  }
  static addTypeId(builder, typeIdOffset) {
    builder.addFieldOffset(2, typeIdOffset, 0);
  }
  static addDefinitionAlias(builder, definitionAliasOffset) {
    builder.addFieldOffset(3, definitionAliasOffset, 0);
  }
  static addDefinitionRevision(builder, definitionRevision) {
    builder.addFieldInt32(4, definitionRevision, 0);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(5, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFormRecord(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormRecord(builder, pathOffset, nameOffset, typeIdOffset, definitionAliasOffset, definitionRevision, valuesOffset) {
    _FormRecord.startFormRecord(builder);
    _FormRecord.addPath(builder, pathOffset);
    _FormRecord.addName(builder, nameOffset);
    _FormRecord.addTypeId(builder, typeIdOffset);
    _FormRecord.addDefinitionAlias(builder, definitionAliasOffset);
    _FormRecord.addDefinitionRevision(builder, definitionRevision);
    _FormRecord.addValues(builder, valuesOffset);
    return _FormRecord.endFormRecord(builder);
  }
  unpack() {
    return new FormRecordT(
      this.path(),
      this.name(),
      this.typeId(),
      this.definitionAlias(),
      this.definitionRevision(),
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.path = this.path();
    _o.name = this.name();
    _o.typeId = this.typeId();
    _o.definitionAlias = this.definitionAlias();
    _o.definitionRevision = this.definitionRevision();
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var FormRecordT = class {
  constructor(path = null, name = null, typeId = null, definitionAlias = null, definitionRevision = 0, values = []) {
    this.path = path;
    this.name = name;
    this.typeId = typeId;
    this.definitionAlias = definitionAlias;
    this.definitionRevision = definitionRevision;
    this.values = values;
  }
  pack(builder) {
    const path = this.path !== null ? builder.createString(this.path) : 0;
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const typeId = this.typeId !== null ? builder.createString(this.typeId) : 0;
    const definitionAlias = this.definitionAlias !== null ? builder.createString(this.definitionAlias) : 0;
    const values = FormRecord.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return FormRecord.createFormRecord(
      builder,
      path,
      name,
      typeId,
      definitionAlias,
      this.definitionRevision,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/form-record-list.ts
var flatbuffers17 = __toESM(require_flatbuffers(), 1);
var FormRecordList = class _FormRecordList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsFormRecordList(bb, obj) {
    return (obj || new _FormRecordList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsFormRecordList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers17.SIZE_PREFIX_LENGTH);
    return (obj || new _FormRecordList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new FormRecord()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startFormRecordList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endFormRecordList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createFormRecordList(builder, valuesOffset) {
    _FormRecordList.startFormRecordList(builder);
    _FormRecordList.addValues(builder, valuesOffset);
    return _FormRecordList.endFormRecordList(builder);
  }
  unpack() {
    return new FormRecordListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var FormRecordListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = FormRecordList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return FormRecordList.createFormRecordList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/import-stats.ts
var flatbuffers18 = __toESM(require_flatbuffers(), 1);
var ImportStats = class _ImportStats {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsImportStats(bb, obj) {
    return (obj || new _ImportStats()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsImportStats(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers18.SIZE_PREFIX_LENGTH);
    return (obj || new _ImportStats()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  hostStatNanos(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  hostReadNanos(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  framePrepareNanos(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  pageWriteNanos(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startImportStats(builder) {
    builder.startObject(4);
  }
  static addHostStatNanos(builder, hostStatNanosOffset) {
    builder.addFieldOffset(0, hostStatNanosOffset, 0);
  }
  static addHostReadNanos(builder, hostReadNanosOffset) {
    builder.addFieldOffset(1, hostReadNanosOffset, 0);
  }
  static addFramePrepareNanos(builder, framePrepareNanosOffset) {
    builder.addFieldOffset(2, framePrepareNanosOffset, 0);
  }
  static addPageWriteNanos(builder, pageWriteNanosOffset) {
    builder.addFieldOffset(3, pageWriteNanosOffset, 0);
  }
  static endImportStats(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createImportStats(builder, hostStatNanosOffset, hostReadNanosOffset, framePrepareNanosOffset, pageWriteNanosOffset) {
    _ImportStats.startImportStats(builder);
    _ImportStats.addHostStatNanos(builder, hostStatNanosOffset);
    _ImportStats.addHostReadNanos(builder, hostReadNanosOffset);
    _ImportStats.addFramePrepareNanos(builder, framePrepareNanosOffset);
    _ImportStats.addPageWriteNanos(builder, pageWriteNanosOffset);
    return _ImportStats.endImportStats(builder);
  }
  unpack() {
    return new ImportStatsT(
      this.hostStatNanos(),
      this.hostReadNanos(),
      this.framePrepareNanos(),
      this.pageWriteNanos()
    );
  }
  unpackTo(_o) {
    _o.hostStatNanos = this.hostStatNanos();
    _o.hostReadNanos = this.hostReadNanos();
    _o.framePrepareNanos = this.framePrepareNanos();
    _o.pageWriteNanos = this.pageWriteNanos();
  }
};
var ImportStatsT = class {
  constructor(hostStatNanos = null, hostReadNanos = null, framePrepareNanos = null, pageWriteNanos = null) {
    this.hostStatNanos = hostStatNanos;
    this.hostReadNanos = hostReadNanos;
    this.framePrepareNanos = framePrepareNanos;
    this.pageWriteNanos = pageWriteNanos;
  }
  pack(builder) {
    const hostStatNanos = this.hostStatNanos !== null ? builder.createString(this.hostStatNanos) : 0;
    const hostReadNanos = this.hostReadNanos !== null ? builder.createString(this.hostReadNanos) : 0;
    const framePrepareNanos = this.framePrepareNanos !== null ? builder.createString(this.framePrepareNanos) : 0;
    const pageWriteNanos = this.pageWriteNanos !== null ? builder.createString(this.pageWriteNanos) : 0;
    return ImportStats.createImportStats(
      builder,
      hostStatNanos,
      hostReadNanos,
      framePrepareNanos,
      pageWriteNanos
    );
  }
};

// generated/flatbuffers/revault/internal/key-slot-list.ts
var flatbuffers19 = __toESM(require_flatbuffers(), 1);
var KeySlotList = class _KeySlotList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsKeySlotList(bb, obj) {
    return (obj || new _KeySlotList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsKeySlotList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers19.SIZE_PREFIX_LENGTH);
    return (obj || new _KeySlotList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new KeySlot()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startKeySlotList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endKeySlotList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createKeySlotList(builder, valuesOffset) {
    _KeySlotList.startKeySlotList(builder);
    _KeySlotList.addValues(builder, valuesOffset);
    return _KeySlotList.endKeySlotList(builder);
  }
  unpack() {
    return new KeySlotListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var KeySlotListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = KeySlotList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return KeySlotList.createKeySlotList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/known-lockbox.ts
var flatbuffers20 = __toESM(require_flatbuffers(), 1);
var KnownLockbox = class _KnownLockbox {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsKnownLockbox(bb, obj) {
    return (obj || new _KnownLockbox()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsKnownLockbox(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers20.SIZE_PREFIX_LENGTH);
    return (obj || new _KnownLockbox()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  lockboxId(index) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  lockboxIdLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  lockboxIdArray() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  path(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  lastSeenUnixMs() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  static startKnownLockbox(builder) {
    builder.startObject(3);
  }
  static addLockboxId(builder, lockboxIdOffset) {
    builder.addFieldOffset(0, lockboxIdOffset, 0);
  }
  static createLockboxIdVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startLockboxIdVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addPath(builder, pathOffset) {
    builder.addFieldOffset(1, pathOffset, 0);
  }
  static addLastSeenUnixMs(builder, lastSeenUnixMs) {
    builder.addFieldInt64(2, lastSeenUnixMs, BigInt("0"));
  }
  static endKnownLockbox(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createKnownLockbox(builder, lockboxIdOffset, pathOffset, lastSeenUnixMs) {
    _KnownLockbox.startKnownLockbox(builder);
    _KnownLockbox.addLockboxId(builder, lockboxIdOffset);
    _KnownLockbox.addPath(builder, pathOffset);
    _KnownLockbox.addLastSeenUnixMs(builder, lastSeenUnixMs);
    return _KnownLockbox.endKnownLockbox(builder);
  }
  unpack() {
    return new KnownLockboxT(
      this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength()),
      this.path(),
      this.lastSeenUnixMs()
    );
  }
  unpackTo(_o) {
    _o.lockboxId = this.bb.createScalarList(this.lockboxId.bind(this), this.lockboxIdLength());
    _o.path = this.path();
    _o.lastSeenUnixMs = this.lastSeenUnixMs();
  }
};
var KnownLockboxT = class {
  constructor(lockboxId = [], path = null, lastSeenUnixMs = BigInt("0")) {
    this.lockboxId = lockboxId;
    this.path = path;
    this.lastSeenUnixMs = lastSeenUnixMs;
  }
  pack(builder) {
    const lockboxId = KnownLockbox.createLockboxIdVector(builder, this.lockboxId);
    const path = this.path !== null ? builder.createString(this.path) : 0;
    return KnownLockbox.createKnownLockbox(
      builder,
      lockboxId,
      path,
      this.lastSeenUnixMs
    );
  }
};

// generated/flatbuffers/revault/internal/known-lockbox-list.ts
var flatbuffers21 = __toESM(require_flatbuffers(), 1);
var KnownLockboxList = class _KnownLockboxList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsKnownLockboxList(bb, obj) {
    return (obj || new _KnownLockboxList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsKnownLockboxList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers21.SIZE_PREFIX_LENGTH);
    return (obj || new _KnownLockboxList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new KnownLockbox()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startKnownLockboxList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endKnownLockboxList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createKnownLockboxList(builder, valuesOffset) {
    _KnownLockboxList.startKnownLockboxList(builder);
    _KnownLockboxList.addValues(builder, valuesOffset);
    return _KnownLockboxList.endKnownLockboxList(builder);
  }
  unpack() {
    return new KnownLockboxListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var KnownLockboxListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = KnownLockboxList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return KnownLockboxList.createKnownLockboxList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/lockbox-entry.ts
var flatbuffers22 = __toESM(require_flatbuffers(), 1);

// generated/flatbuffers/revault/internal/lockbox-entry-kind.ts
var LockboxEntryKind = /* @__PURE__ */ ((LockboxEntryKind2) => {
  LockboxEntryKind2[LockboxEntryKind2["KIND_UNSPECIFIED"] = 0] = "KIND_UNSPECIFIED";
  LockboxEntryKind2[LockboxEntryKind2["FILE"] = 1] = "FILE";
  LockboxEntryKind2[LockboxEntryKind2["SYMLINK"] = 2] = "SYMLINK";
  LockboxEntryKind2[LockboxEntryKind2["DIRECTORY"] = 3] = "DIRECTORY";
  return LockboxEntryKind2;
})(LockboxEntryKind || {});

// generated/flatbuffers/revault/internal/lockbox-entry.ts
var LockboxEntry = class _LockboxEntry {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsLockboxEntry(bb, obj) {
    return (obj || new _LockboxEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsLockboxEntry(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers22.SIZE_PREFIX_LENGTH);
    return (obj || new _LockboxEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  path(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  kind() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0 /* KIND_UNSPECIFIED */;
  }
  length() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  permissions() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  static startLockboxEntry(builder) {
    builder.startObject(4);
  }
  static addPath(builder, pathOffset) {
    builder.addFieldOffset(0, pathOffset, 0);
  }
  static addKind(builder, kind) {
    builder.addFieldInt32(1, kind, 0 /* KIND_UNSPECIFIED */);
  }
  static addLength(builder, length) {
    builder.addFieldInt64(2, length, BigInt("0"));
  }
  static addPermissions(builder, permissions) {
    builder.addFieldInt32(3, permissions, 0);
  }
  static endLockboxEntry(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createLockboxEntry(builder, pathOffset, kind, length, permissions) {
    _LockboxEntry.startLockboxEntry(builder);
    _LockboxEntry.addPath(builder, pathOffset);
    _LockboxEntry.addKind(builder, kind);
    _LockboxEntry.addLength(builder, length);
    _LockboxEntry.addPermissions(builder, permissions);
    return _LockboxEntry.endLockboxEntry(builder);
  }
  unpack() {
    return new LockboxEntryT(
      this.path(),
      this.kind(),
      this.length(),
      this.permissions()
    );
  }
  unpackTo(_o) {
    _o.path = this.path();
    _o.kind = this.kind();
    _o.length = this.length();
    _o.permissions = this.permissions();
  }
};
var LockboxEntryT = class {
  constructor(path = null, kind = 0 /* KIND_UNSPECIFIED */, length = BigInt("0"), permissions = 0) {
    this.path = path;
    this.kind = kind;
    this.length = length;
    this.permissions = permissions;
  }
  pack(builder) {
    const path = this.path !== null ? builder.createString(this.path) : 0;
    return LockboxEntry.createLockboxEntry(
      builder,
      path,
      this.kind,
      this.length,
      this.permissions
    );
  }
};

// generated/flatbuffers/revault/internal/lockbox-entry-list.ts
var flatbuffers23 = __toESM(require_flatbuffers(), 1);
var LockboxEntryList = class _LockboxEntryList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsLockboxEntryList(bb, obj) {
    return (obj || new _LockboxEntryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsLockboxEntryList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers23.SIZE_PREFIX_LENGTH);
    return (obj || new _LockboxEntryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  entries(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new LockboxEntry()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  entriesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startLockboxEntryList(builder) {
    builder.startObject(1);
  }
  static addEntries(builder, entriesOffset) {
    builder.addFieldOffset(0, entriesOffset, 0);
  }
  static createEntriesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startEntriesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endLockboxEntryList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createLockboxEntryList(builder, entriesOffset) {
    _LockboxEntryList.startLockboxEntryList(builder);
    _LockboxEntryList.addEntries(builder, entriesOffset);
    return _LockboxEntryList.endLockboxEntryList(builder);
  }
  unpack() {
    return new LockboxEntryListT(
      this.bb.createObjList(this.entries.bind(this), this.entriesLength())
    );
  }
  unpackTo(_o) {
    _o.entries = this.bb.createObjList(this.entries.bind(this), this.entriesLength());
  }
};
var LockboxEntryListT = class {
  constructor(entries = []) {
    this.entries = entries;
  }
  pack(builder) {
    const entries = LockboxEntryList.createEntriesVector(builder, builder.createObjectOffsetList(this.entries));
    return LockboxEntryList.createLockboxEntryList(
      builder,
      entries
    );
  }
};

// generated/flatbuffers/revault/internal/optional-form-record.ts
var flatbuffers24 = __toESM(require_flatbuffers(), 1);
var OptionalFormRecord = class _OptionalFormRecord {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsOptionalFormRecord(bb, obj) {
    return (obj || new _OptionalFormRecord()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsOptionalFormRecord(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers24.SIZE_PREFIX_LENGTH);
    return (obj || new _OptionalFormRecord()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  value(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new FormRecord()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static startOptionalFormRecord(builder) {
    builder.startObject(1);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(0, valueOffset, 0);
  }
  static endOptionalFormRecord(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createOptionalFormRecord(builder, valueOffset) {
    _OptionalFormRecord.startOptionalFormRecord(builder);
    _OptionalFormRecord.addValue(builder, valueOffset);
    return _OptionalFormRecord.endOptionalFormRecord(builder);
  }
  unpack() {
    return new OptionalFormRecordT(
      this.value() !== null ? this.value().unpack() : null
    );
  }
  unpackTo(_o) {
    _o.value = this.value() !== null ? this.value().unpack() : null;
  }
};
var OptionalFormRecordT = class {
  constructor(value = null) {
    this.value = value;
  }
  pack(builder) {
    const value = this.value !== null ? this.value.pack(builder) : 0;
    return OptionalFormRecord.createOptionalFormRecord(
      builder,
      value
    );
  }
};

// generated/flatbuffers/revault/internal/optional-form-value.ts
var flatbuffers25 = __toESM(require_flatbuffers(), 1);
var OptionalFormValue = class _OptionalFormValue {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsOptionalFormValue(bb, obj) {
    return (obj || new _OptionalFormValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsOptionalFormValue(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers25.SIZE_PREFIX_LENGTH);
    return (obj || new _OptionalFormValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  value(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new FormValue()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static startOptionalFormValue(builder) {
    builder.startObject(1);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(0, valueOffset, 0);
  }
  static endOptionalFormValue(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createOptionalFormValue(builder, valueOffset) {
    _OptionalFormValue.startOptionalFormValue(builder);
    _OptionalFormValue.addValue(builder, valueOffset);
    return _OptionalFormValue.endOptionalFormValue(builder);
  }
  unpack() {
    return new OptionalFormValueT(
      this.value() !== null ? this.value().unpack() : null
    );
  }
  unpackTo(_o) {
    _o.value = this.value() !== null ? this.value().unpack() : null;
  }
};
var OptionalFormValueT = class {
  constructor(value = null) {
    this.value = value;
  }
  pack(builder) {
    const value = this.value !== null ? this.value.pack(builder) : 0;
    return OptionalFormValue.createOptionalFormValue(
      builder,
      value
    );
  }
};

// generated/flatbuffers/revault/internal/optional-lockbox-entry.ts
var flatbuffers26 = __toESM(require_flatbuffers(), 1);
var OptionalLockboxEntry = class _OptionalLockboxEntry {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsOptionalLockboxEntry(bb, obj) {
    return (obj || new _OptionalLockboxEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsOptionalLockboxEntry(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers26.SIZE_PREFIX_LENGTH);
    return (obj || new _OptionalLockboxEntry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  value(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new LockboxEntry()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static startOptionalLockboxEntry(builder) {
    builder.startObject(1);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(0, valueOffset, 0);
  }
  static endOptionalLockboxEntry(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createOptionalLockboxEntry(builder, valueOffset) {
    _OptionalLockboxEntry.startOptionalLockboxEntry(builder);
    _OptionalLockboxEntry.addValue(builder, valueOffset);
    return _OptionalLockboxEntry.endOptionalLockboxEntry(builder);
  }
  unpack() {
    return new OptionalLockboxEntryT(
      this.value() !== null ? this.value().unpack() : null
    );
  }
  unpackTo(_o) {
    _o.value = this.value() !== null ? this.value().unpack() : null;
  }
};
var OptionalLockboxEntryT = class {
  constructor(value = null) {
    this.value = value;
  }
  pack(builder) {
    const value = this.value !== null ? this.value.pack(builder) : 0;
    return OptionalLockboxEntry.createOptionalLockboxEntry(
      builder,
      value
    );
  }
};

// generated/flatbuffers/revault/internal/optional-string.ts
var flatbuffers27 = __toESM(require_flatbuffers(), 1);
var OptionalString = class _OptionalString {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsOptionalString(bb, obj) {
    return (obj || new _OptionalString()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsOptionalString(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers27.SIZE_PREFIX_LENGTH);
    return (obj || new _OptionalString()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  present() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  value(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startOptionalString(builder) {
    builder.startObject(2);
  }
  static addPresent(builder, present) {
    builder.addFieldInt8(0, +present, 0);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(1, valueOffset, 0);
  }
  static endOptionalString(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createOptionalString(builder, present, valueOffset) {
    _OptionalString.startOptionalString(builder);
    _OptionalString.addPresent(builder, present);
    _OptionalString.addValue(builder, valueOffset);
    return _OptionalString.endOptionalString(builder);
  }
  unpack() {
    return new OptionalStringT(
      this.present(),
      this.value()
    );
  }
  unpackTo(_o) {
    _o.present = this.present();
    _o.value = this.value();
  }
};
var OptionalStringT = class {
  constructor(present = false, value = null) {
    this.present = present;
    this.value = value;
  }
  pack(builder) {
    const value = this.value !== null ? builder.createString(this.value) : 0;
    return OptionalString.createOptionalString(
      builder,
      this.present,
      value
    );
  }
};

// generated/flatbuffers/revault/internal/owner-inspection.ts
var flatbuffers28 = __toESM(require_flatbuffers(), 1);
var OwnerInspection = class _OwnerInspection {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsOwnerInspection(bb, obj) {
    return (obj || new _OwnerInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsOwnerInspection(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers28.SIZE_PREFIX_LENGTH);
    return (obj || new _OwnerInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  signed() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  fingerprint(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  hasFingerprint() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  static startOwnerInspection(builder) {
    builder.startObject(3);
  }
  static addSigned(builder, signed) {
    builder.addFieldInt8(0, +signed, 0);
  }
  static addFingerprint(builder, fingerprintOffset) {
    builder.addFieldOffset(1, fingerprintOffset, 0);
  }
  static addHasFingerprint(builder, hasFingerprint) {
    builder.addFieldInt8(2, +hasFingerprint, 0);
  }
  static endOwnerInspection(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createOwnerInspection(builder, signed, fingerprintOffset, hasFingerprint) {
    _OwnerInspection.startOwnerInspection(builder);
    _OwnerInspection.addSigned(builder, signed);
    _OwnerInspection.addFingerprint(builder, fingerprintOffset);
    _OwnerInspection.addHasFingerprint(builder, hasFingerprint);
    return _OwnerInspection.endOwnerInspection(builder);
  }
  unpack() {
    return new OwnerInspectionT(
      this.signed(),
      this.fingerprint(),
      this.hasFingerprint()
    );
  }
  unpackTo(_o) {
    _o.signed = this.signed();
    _o.fingerprint = this.fingerprint();
    _o.hasFingerprint = this.hasFingerprint();
  }
};
var OwnerInspectionT = class {
  constructor(signed = false, fingerprint = null, hasFingerprint = false) {
    this.signed = signed;
    this.fingerprint = fingerprint;
    this.hasFingerprint = hasFingerprint;
  }
  pack(builder) {
    const fingerprint = this.fingerprint !== null ? builder.createString(this.fingerprint) : 0;
    return OwnerInspection.createOwnerInspection(
      builder,
      this.signed,
      fingerprint,
      this.hasFingerprint
    );
  }
};

// generated/flatbuffers/revault/internal/page-inspection.ts
var flatbuffers30 = __toESM(require_flatbuffers(), 1);

// generated/flatbuffers/revault/internal/page-object.ts
var flatbuffers29 = __toESM(require_flatbuffers(), 1);
var PageObject = class _PageObject {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPageObject(bb, obj) {
    return (obj || new _PageObject()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPageObject(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers29.SIZE_PREFIX_LENGTH);
    return (obj || new _PageObject()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  kind(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  payloadLen() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  static startPageObject(builder) {
    builder.startObject(3);
  }
  static addId(builder, id) {
    builder.addFieldInt64(0, id, BigInt("0"));
  }
  static addKind(builder, kindOffset) {
    builder.addFieldOffset(1, kindOffset, 0);
  }
  static addPayloadLen(builder, payloadLen) {
    builder.addFieldInt64(2, payloadLen, BigInt("0"));
  }
  static endPageObject(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPageObject(builder, id, kindOffset, payloadLen) {
    _PageObject.startPageObject(builder);
    _PageObject.addId(builder, id);
    _PageObject.addKind(builder, kindOffset);
    _PageObject.addPayloadLen(builder, payloadLen);
    return _PageObject.endPageObject(builder);
  }
  unpack() {
    return new PageObjectT(
      this.id(),
      this.kind(),
      this.payloadLen()
    );
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.kind = this.kind();
    _o.payloadLen = this.payloadLen();
  }
};
var PageObjectT = class {
  constructor(id = BigInt("0"), kind = null, payloadLen = BigInt("0")) {
    this.id = id;
    this.kind = kind;
    this.payloadLen = payloadLen;
  }
  pack(builder) {
    const kind = this.kind !== null ? builder.createString(this.kind) : 0;
    return PageObject.createPageObject(
      builder,
      this.id,
      kind,
      this.payloadLen
    );
  }
};

// generated/flatbuffers/revault/internal/page-inspection.ts
var PageInspection = class _PageInspection {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPageInspection(bb, obj) {
    return (obj || new _PageInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPageInspection(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers30.SIZE_PREFIX_LENGTH);
    return (obj || new _PageInspection()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  offset() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  pageId() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  sequence() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  pageSize() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  encryptedBodyLen() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  unusedBytes() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  objectCount() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  objects(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? (obj || new PageObject()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  objectsLength() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startPageInspection(builder) {
    builder.startObject(8);
  }
  static addOffset(builder, offset) {
    builder.addFieldInt64(0, offset, BigInt("0"));
  }
  static addPageId(builder, pageId) {
    builder.addFieldInt64(1, pageId, BigInt("0"));
  }
  static addSequence(builder, sequence) {
    builder.addFieldInt64(2, sequence, BigInt("0"));
  }
  static addPageSize(builder, pageSize) {
    builder.addFieldInt64(3, pageSize, BigInt("0"));
  }
  static addEncryptedBodyLen(builder, encryptedBodyLen) {
    builder.addFieldInt64(4, encryptedBodyLen, BigInt("0"));
  }
  static addUnusedBytes(builder, unusedBytes) {
    builder.addFieldInt64(5, unusedBytes, BigInt("0"));
  }
  static addObjectCount(builder, objectCount) {
    builder.addFieldInt64(6, objectCount, BigInt("0"));
  }
  static addObjects(builder, objectsOffset) {
    builder.addFieldOffset(7, objectsOffset, 0);
  }
  static createObjectsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startObjectsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endPageInspection(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPageInspection(builder, offset, pageId, sequence, pageSize, encryptedBodyLen, unusedBytes, objectCount, objectsOffset) {
    _PageInspection.startPageInspection(builder);
    _PageInspection.addOffset(builder, offset);
    _PageInspection.addPageId(builder, pageId);
    _PageInspection.addSequence(builder, sequence);
    _PageInspection.addPageSize(builder, pageSize);
    _PageInspection.addEncryptedBodyLen(builder, encryptedBodyLen);
    _PageInspection.addUnusedBytes(builder, unusedBytes);
    _PageInspection.addObjectCount(builder, objectCount);
    _PageInspection.addObjects(builder, objectsOffset);
    return _PageInspection.endPageInspection(builder);
  }
  unpack() {
    return new PageInspectionT(
      this.offset(),
      this.pageId(),
      this.sequence(),
      this.pageSize(),
      this.encryptedBodyLen(),
      this.unusedBytes(),
      this.objectCount(),
      this.bb.createObjList(this.objects.bind(this), this.objectsLength())
    );
  }
  unpackTo(_o) {
    _o.offset = this.offset();
    _o.pageId = this.pageId();
    _o.sequence = this.sequence();
    _o.pageSize = this.pageSize();
    _o.encryptedBodyLen = this.encryptedBodyLen();
    _o.unusedBytes = this.unusedBytes();
    _o.objectCount = this.objectCount();
    _o.objects = this.bb.createObjList(this.objects.bind(this), this.objectsLength());
  }
};
var PageInspectionT = class {
  constructor(offset = BigInt("0"), pageId = BigInt("0"), sequence = BigInt("0"), pageSize = BigInt("0"), encryptedBodyLen = BigInt("0"), unusedBytes = BigInt("0"), objectCount = BigInt("0"), objects = []) {
    this.offset = offset;
    this.pageId = pageId;
    this.sequence = sequence;
    this.pageSize = pageSize;
    this.encryptedBodyLen = encryptedBodyLen;
    this.unusedBytes = unusedBytes;
    this.objectCount = objectCount;
    this.objects = objects;
  }
  pack(builder) {
    const objects = PageInspection.createObjectsVector(builder, builder.createObjectOffsetList(this.objects));
    return PageInspection.createPageInspection(
      builder,
      this.offset,
      this.pageId,
      this.sequence,
      this.pageSize,
      this.encryptedBodyLen,
      this.unusedBytes,
      this.objectCount,
      objects
    );
  }
};

// generated/flatbuffers/revault/internal/page-inspection-list.ts
var flatbuffers31 = __toESM(require_flatbuffers(), 1);
var PageInspectionList = class _PageInspectionList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPageInspectionList(bb, obj) {
    return (obj || new _PageInspectionList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPageInspectionList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers31.SIZE_PREFIX_LENGTH);
    return (obj || new _PageInspectionList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new PageInspection()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startPageInspectionList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endPageInspectionList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPageInspectionList(builder, valuesOffset) {
    _PageInspectionList.startPageInspectionList(builder);
    _PageInspectionList.addValues(builder, valuesOffset);
    return _PageInspectionList.endPageInspectionList(builder);
  }
  unpack() {
    return new PageInspectionListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var PageInspectionListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = PageInspectionList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return PageInspectionList.createPageInspectionList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/path-move.ts
var flatbuffers32 = __toESM(require_flatbuffers(), 1);
var PathMove = class _PathMove {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPathMove(bb, obj) {
    return (obj || new _PathMove()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPathMove(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers32.SIZE_PREFIX_LENGTH);
    return (obj || new _PathMove()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  source(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  destination(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startPathMove(builder) {
    builder.startObject(2);
  }
  static addSource(builder, sourceOffset) {
    builder.addFieldOffset(0, sourceOffset, 0);
  }
  static addDestination(builder, destinationOffset) {
    builder.addFieldOffset(1, destinationOffset, 0);
  }
  static endPathMove(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPathMove(builder, sourceOffset, destinationOffset) {
    _PathMove.startPathMove(builder);
    _PathMove.addSource(builder, sourceOffset);
    _PathMove.addDestination(builder, destinationOffset);
    return _PathMove.endPathMove(builder);
  }
  unpack() {
    return new PathMoveT(
      this.source(),
      this.destination()
    );
  }
  unpackTo(_o) {
    _o.source = this.source();
    _o.destination = this.destination();
  }
};
var PathMoveT = class {
  constructor(source = null, destination = null) {
    this.source = source;
    this.destination = destination;
  }
  pack(builder) {
    const source = this.source !== null ? builder.createString(this.source) : 0;
    const destination = this.destination !== null ? builder.createString(this.destination) : 0;
    return PathMove.createPathMove(
      builder,
      source,
      destination
    );
  }
};

// generated/flatbuffers/revault/internal/path-move-list.ts
var flatbuffers33 = __toESM(require_flatbuffers(), 1);
var PathMoveList = class _PathMoveList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPathMoveList(bb, obj) {
    return (obj || new _PathMoveList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPathMoveList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers33.SIZE_PREFIX_LENGTH);
    return (obj || new _PathMoveList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new PathMove()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startPathMoveList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endPathMoveList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPathMoveList(builder, valuesOffset) {
    _PathMoveList.startPathMoveList(builder);
    _PathMoveList.addValues(builder, valuesOffset);
    return _PathMoveList.endPathMoveList(builder);
  }
  unpack() {
    return new PathMoveListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var PathMoveListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = PathMoveList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return PathMoveList.createPathMoveList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/platform-status.ts
var flatbuffers34 = __toESM(require_flatbuffers(), 1);
var PlatformStatus = class _PlatformStatus {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsPlatformStatus(bb, obj) {
    return (obj || new _PlatformStatus()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsPlatformStatus(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers34.SIZE_PREFIX_LENGTH);
    return (obj || new _PlatformStatus()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  supported() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  disabled() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  scope(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  backend(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  item(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startPlatformStatus(builder) {
    builder.startObject(5);
  }
  static addSupported(builder, supported) {
    builder.addFieldInt8(0, +supported, 0);
  }
  static addDisabled(builder, disabled) {
    builder.addFieldInt8(1, +disabled, 0);
  }
  static addScope(builder, scopeOffset) {
    builder.addFieldOffset(2, scopeOffset, 0);
  }
  static addBackend(builder, backendOffset) {
    builder.addFieldOffset(3, backendOffset, 0);
  }
  static addItem(builder, itemOffset) {
    builder.addFieldOffset(4, itemOffset, 0);
  }
  static endPlatformStatus(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createPlatformStatus(builder, supported, disabled, scopeOffset, backendOffset, itemOffset) {
    _PlatformStatus.startPlatformStatus(builder);
    _PlatformStatus.addSupported(builder, supported);
    _PlatformStatus.addDisabled(builder, disabled);
    _PlatformStatus.addScope(builder, scopeOffset);
    _PlatformStatus.addBackend(builder, backendOffset);
    _PlatformStatus.addItem(builder, itemOffset);
    return _PlatformStatus.endPlatformStatus(builder);
  }
  unpack() {
    return new PlatformStatusT(
      this.supported(),
      this.disabled(),
      this.scope(),
      this.backend(),
      this.item()
    );
  }
  unpackTo(_o) {
    _o.supported = this.supported();
    _o.disabled = this.disabled();
    _o.scope = this.scope();
    _o.backend = this.backend();
    _o.item = this.item();
  }
};
var PlatformStatusT = class {
  constructor(supported = false, disabled = false, scope = null, backend = null, item = null) {
    this.supported = supported;
    this.disabled = disabled;
    this.scope = scope;
    this.backend = backend;
    this.item = item;
  }
  pack(builder) {
    const scope = this.scope !== null ? builder.createString(this.scope) : 0;
    const backend = this.backend !== null ? builder.createString(this.backend) : 0;
    const item = this.item !== null ? builder.createString(this.item) : 0;
    return PlatformStatus.createPlatformStatus(
      builder,
      this.supported,
      this.disabled,
      scope,
      backend,
      item
    );
  }
};

// generated/flatbuffers/revault/internal/profile-generation.ts
var flatbuffers35 = __toESM(require_flatbuffers(), 1);
var ProfileGeneration = class _ProfileGeneration {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsProfileGeneration(bb, obj) {
    return (obj || new _ProfileGeneration()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsProfileGeneration(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers35.SIZE_PREFIX_LENGTH);
    return (obj || new _ProfileGeneration()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  index() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  status(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  contactFingerprint(index) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  contactFingerprintLength() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  contactFingerprintArray() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  createdAtUnixMs() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  retiredAtUnixMs() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  hasRetiredAt() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  static startProfileGeneration(builder) {
    builder.startObject(6);
  }
  static addIndex(builder, index) {
    builder.addFieldInt32(0, index, 0);
  }
  static addStatus(builder, statusOffset) {
    builder.addFieldOffset(1, statusOffset, 0);
  }
  static addContactFingerprint(builder, contactFingerprintOffset) {
    builder.addFieldOffset(2, contactFingerprintOffset, 0);
  }
  static createContactFingerprintVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startContactFingerprintVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addCreatedAtUnixMs(builder, createdAtUnixMs) {
    builder.addFieldInt64(3, createdAtUnixMs, BigInt("0"));
  }
  static addRetiredAtUnixMs(builder, retiredAtUnixMs) {
    builder.addFieldInt64(4, retiredAtUnixMs, BigInt("0"));
  }
  static addHasRetiredAt(builder, hasRetiredAt) {
    builder.addFieldInt8(5, +hasRetiredAt, 0);
  }
  static endProfileGeneration(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createProfileGeneration(builder, index, statusOffset, contactFingerprintOffset, createdAtUnixMs, retiredAtUnixMs, hasRetiredAt) {
    _ProfileGeneration.startProfileGeneration(builder);
    _ProfileGeneration.addIndex(builder, index);
    _ProfileGeneration.addStatus(builder, statusOffset);
    _ProfileGeneration.addContactFingerprint(builder, contactFingerprintOffset);
    _ProfileGeneration.addCreatedAtUnixMs(builder, createdAtUnixMs);
    _ProfileGeneration.addRetiredAtUnixMs(builder, retiredAtUnixMs);
    _ProfileGeneration.addHasRetiredAt(builder, hasRetiredAt);
    return _ProfileGeneration.endProfileGeneration(builder);
  }
  unpack() {
    return new ProfileGenerationT(
      this.index(),
      this.status(),
      this.bb.createScalarList(this.contactFingerprint.bind(this), this.contactFingerprintLength()),
      this.createdAtUnixMs(),
      this.retiredAtUnixMs(),
      this.hasRetiredAt()
    );
  }
  unpackTo(_o) {
    _o.index = this.index();
    _o.status = this.status();
    _o.contactFingerprint = this.bb.createScalarList(this.contactFingerprint.bind(this), this.contactFingerprintLength());
    _o.createdAtUnixMs = this.createdAtUnixMs();
    _o.retiredAtUnixMs = this.retiredAtUnixMs();
    _o.hasRetiredAt = this.hasRetiredAt();
  }
};
var ProfileGenerationT = class {
  constructor(index = 0, status = null, contactFingerprint = [], createdAtUnixMs = BigInt("0"), retiredAtUnixMs = BigInt("0"), hasRetiredAt = false) {
    this.index = index;
    this.status = status;
    this.contactFingerprint = contactFingerprint;
    this.createdAtUnixMs = createdAtUnixMs;
    this.retiredAtUnixMs = retiredAtUnixMs;
    this.hasRetiredAt = hasRetiredAt;
  }
  pack(builder) {
    const status = this.status !== null ? builder.createString(this.status) : 0;
    const contactFingerprint = ProfileGeneration.createContactFingerprintVector(builder, this.contactFingerprint);
    return ProfileGeneration.createProfileGeneration(
      builder,
      this.index,
      status,
      contactFingerprint,
      this.createdAtUnixMs,
      this.retiredAtUnixMs,
      this.hasRetiredAt
    );
  }
};

// generated/flatbuffers/revault/internal/profile-history.ts
var flatbuffers36 = __toESM(require_flatbuffers(), 1);
var ProfileHistory = class _ProfileHistory {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsProfileHistory(bb, obj) {
    return (obj || new _ProfileHistory()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsProfileHistory(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers36.SIZE_PREFIX_LENGTH);
    return (obj || new _ProfileHistory()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  activeGeneration() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  generations(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? (obj || new ProfileGeneration()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  generationsLength() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startProfileHistory(builder) {
    builder.startObject(3);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addActiveGeneration(builder, activeGeneration) {
    builder.addFieldInt32(1, activeGeneration, 0);
  }
  static addGenerations(builder, generationsOffset) {
    builder.addFieldOffset(2, generationsOffset, 0);
  }
  static createGenerationsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startGenerationsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endProfileHistory(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createProfileHistory(builder, nameOffset, activeGeneration, generationsOffset) {
    _ProfileHistory.startProfileHistory(builder);
    _ProfileHistory.addName(builder, nameOffset);
    _ProfileHistory.addActiveGeneration(builder, activeGeneration);
    _ProfileHistory.addGenerations(builder, generationsOffset);
    return _ProfileHistory.endProfileHistory(builder);
  }
  unpack() {
    return new ProfileHistoryT(
      this.name(),
      this.activeGeneration(),
      this.bb.createObjList(this.generations.bind(this), this.generationsLength())
    );
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.activeGeneration = this.activeGeneration();
    _o.generations = this.bb.createObjList(this.generations.bind(this), this.generationsLength());
  }
};
var ProfileHistoryT = class {
  constructor(name = null, activeGeneration = 0, generations = []) {
    this.name = name;
    this.activeGeneration = activeGeneration;
    this.generations = generations;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const generations = ProfileHistory.createGenerationsVector(builder, builder.createObjectOffsetList(this.generations));
    return ProfileHistory.createProfileHistory(
      builder,
      name,
      this.activeGeneration,
      generations
    );
  }
};

// generated/flatbuffers/revault/internal/profile-history-list.ts
var flatbuffers37 = __toESM(require_flatbuffers(), 1);
var ProfileHistoryList = class _ProfileHistoryList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsProfileHistoryList(bb, obj) {
    return (obj || new _ProfileHistoryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsProfileHistoryList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers37.SIZE_PREFIX_LENGTH);
    return (obj || new _ProfileHistoryList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new ProfileHistory()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startProfileHistoryList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endProfileHistoryList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createProfileHistoryList(builder, valuesOffset) {
    _ProfileHistoryList.startProfileHistoryList(builder);
    _ProfileHistoryList.addValues(builder, valuesOffset);
    return _ProfileHistoryList.endProfileHistoryList(builder);
  }
  unpack() {
    return new ProfileHistoryListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var ProfileHistoryListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = ProfileHistoryList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return ProfileHistoryList.createProfileHistoryList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/recovery-report.ts
var flatbuffers38 = __toESM(require_flatbuffers(), 1);
var RecoveryReport = class _RecoveryReport {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsRecoveryReport(bb, obj) {
    return (obj || new _RecoveryReport()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsRecoveryReport(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers38.SIZE_PREFIX_LENGTH);
    return (obj || new _RecoveryReport()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  intactFiles(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new LockboxEntry()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  intactFilesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  intactFileCount() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  partialFiles() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  corruptRecords() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  tocRecovered() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  variablesRecovered() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  variableCount() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  formsRecovered() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  formDefinitionCount() {
    const offset = this.bb.__offset(this.bb_pos, 20);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  formRecordCount() {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  static startRecoveryReport(builder) {
    builder.startObject(10);
  }
  static addIntactFiles(builder, intactFilesOffset) {
    builder.addFieldOffset(0, intactFilesOffset, 0);
  }
  static createIntactFilesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startIntactFilesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addIntactFileCount(builder, intactFileCount) {
    builder.addFieldInt64(1, intactFileCount, BigInt("0"));
  }
  static addPartialFiles(builder, partialFiles) {
    builder.addFieldInt64(2, partialFiles, BigInt("0"));
  }
  static addCorruptRecords(builder, corruptRecords) {
    builder.addFieldInt64(3, corruptRecords, BigInt("0"));
  }
  static addTocRecovered(builder, tocRecovered) {
    builder.addFieldInt8(4, +tocRecovered, 0);
  }
  static addVariablesRecovered(builder, variablesRecovered) {
    builder.addFieldInt8(5, +variablesRecovered, 0);
  }
  static addVariableCount(builder, variableCount) {
    builder.addFieldInt64(6, variableCount, BigInt("0"));
  }
  static addFormsRecovered(builder, formsRecovered) {
    builder.addFieldInt8(7, +formsRecovered, 0);
  }
  static addFormDefinitionCount(builder, formDefinitionCount) {
    builder.addFieldInt64(8, formDefinitionCount, BigInt("0"));
  }
  static addFormRecordCount(builder, formRecordCount) {
    builder.addFieldInt64(9, formRecordCount, BigInt("0"));
  }
  static endRecoveryReport(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createRecoveryReport(builder, intactFilesOffset, intactFileCount, partialFiles, corruptRecords, tocRecovered, variablesRecovered, variableCount, formsRecovered, formDefinitionCount, formRecordCount) {
    _RecoveryReport.startRecoveryReport(builder);
    _RecoveryReport.addIntactFiles(builder, intactFilesOffset);
    _RecoveryReport.addIntactFileCount(builder, intactFileCount);
    _RecoveryReport.addPartialFiles(builder, partialFiles);
    _RecoveryReport.addCorruptRecords(builder, corruptRecords);
    _RecoveryReport.addTocRecovered(builder, tocRecovered);
    _RecoveryReport.addVariablesRecovered(builder, variablesRecovered);
    _RecoveryReport.addVariableCount(builder, variableCount);
    _RecoveryReport.addFormsRecovered(builder, formsRecovered);
    _RecoveryReport.addFormDefinitionCount(builder, formDefinitionCount);
    _RecoveryReport.addFormRecordCount(builder, formRecordCount);
    return _RecoveryReport.endRecoveryReport(builder);
  }
  unpack() {
    return new RecoveryReportT(
      this.bb.createObjList(this.intactFiles.bind(this), this.intactFilesLength()),
      this.intactFileCount(),
      this.partialFiles(),
      this.corruptRecords(),
      this.tocRecovered(),
      this.variablesRecovered(),
      this.variableCount(),
      this.formsRecovered(),
      this.formDefinitionCount(),
      this.formRecordCount()
    );
  }
  unpackTo(_o) {
    _o.intactFiles = this.bb.createObjList(this.intactFiles.bind(this), this.intactFilesLength());
    _o.intactFileCount = this.intactFileCount();
    _o.partialFiles = this.partialFiles();
    _o.corruptRecords = this.corruptRecords();
    _o.tocRecovered = this.tocRecovered();
    _o.variablesRecovered = this.variablesRecovered();
    _o.variableCount = this.variableCount();
    _o.formsRecovered = this.formsRecovered();
    _o.formDefinitionCount = this.formDefinitionCount();
    _o.formRecordCount = this.formRecordCount();
  }
};
var RecoveryReportT = class {
  constructor(intactFiles = [], intactFileCount = BigInt("0"), partialFiles = BigInt("0"), corruptRecords = BigInt("0"), tocRecovered = false, variablesRecovered = false, variableCount = BigInt("0"), formsRecovered = false, formDefinitionCount = BigInt("0"), formRecordCount = BigInt("0")) {
    this.intactFiles = intactFiles;
    this.intactFileCount = intactFileCount;
    this.partialFiles = partialFiles;
    this.corruptRecords = corruptRecords;
    this.tocRecovered = tocRecovered;
    this.variablesRecovered = variablesRecovered;
    this.variableCount = variableCount;
    this.formsRecovered = formsRecovered;
    this.formDefinitionCount = formDefinitionCount;
    this.formRecordCount = formRecordCount;
  }
  pack(builder) {
    const intactFiles = RecoveryReport.createIntactFilesVector(builder, builder.createObjectOffsetList(this.intactFiles));
    return RecoveryReport.createRecoveryReport(
      builder,
      intactFiles,
      this.intactFileCount,
      this.partialFiles,
      this.corruptRecords,
      this.tocRecovered,
      this.variablesRecovered,
      this.variableCount,
      this.formsRecovered,
      this.formDefinitionCount,
      this.formRecordCount
    );
  }
};

// generated/flatbuffers/revault/internal/runtime-options.ts
var flatbuffers39 = __toESM(require_flatbuffers(), 1);
var RuntimeOptions = class _RuntimeOptions {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsRuntimeOptions(bb, obj) {
    return (obj || new _RuntimeOptions()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsRuntimeOptions(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers39.SIZE_PREFIX_LENGTH);
    return (obj || new _RuntimeOptions()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  workloadProfile(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  workerPolicy(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startRuntimeOptions(builder) {
    builder.startObject(2);
  }
  static addWorkloadProfile(builder, workloadProfileOffset) {
    builder.addFieldOffset(0, workloadProfileOffset, 0);
  }
  static addWorkerPolicy(builder, workerPolicyOffset) {
    builder.addFieldOffset(1, workerPolicyOffset, 0);
  }
  static endRuntimeOptions(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createRuntimeOptions(builder, workloadProfileOffset, workerPolicyOffset) {
    _RuntimeOptions.startRuntimeOptions(builder);
    _RuntimeOptions.addWorkloadProfile(builder, workloadProfileOffset);
    _RuntimeOptions.addWorkerPolicy(builder, workerPolicyOffset);
    return _RuntimeOptions.endRuntimeOptions(builder);
  }
  unpack() {
    return new RuntimeOptionsT(
      this.workloadProfile(),
      this.workerPolicy()
    );
  }
  unpackTo(_o) {
    _o.workloadProfile = this.workloadProfile();
    _o.workerPolicy = this.workerPolicy();
  }
};
var RuntimeOptionsT = class {
  constructor(workloadProfile = null, workerPolicy = null) {
    this.workloadProfile = workloadProfile;
    this.workerPolicy = workerPolicy;
  }
  pack(builder) {
    const workloadProfile = this.workloadProfile !== null ? builder.createString(this.workloadProfile) : 0;
    const workerPolicy = this.workerPolicy !== null ? builder.createString(this.workerPolicy) : 0;
    return RuntimeOptions.createRuntimeOptions(
      builder,
      workloadProfile,
      workerPolicy
    );
  }
};

// generated/flatbuffers/revault/internal/sleep-support.ts
var flatbuffers40 = __toESM(require_flatbuffers(), 1);
var SleepSupport = class _SleepSupport {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsSleepSupport(bb, obj) {
    return (obj || new _SleepSupport()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsSleepSupport(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers40.SIZE_PREFIX_LENGTH);
    return (obj || new _SleepSupport()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  suspendNotifications() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  sleepInhibition() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  supported() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  static startSleepSupport(builder) {
    builder.startObject(3);
  }
  static addSuspendNotifications(builder, suspendNotifications) {
    builder.addFieldInt8(0, +suspendNotifications, 0);
  }
  static addSleepInhibition(builder, sleepInhibition) {
    builder.addFieldInt8(1, +sleepInhibition, 0);
  }
  static addSupported(builder, supported) {
    builder.addFieldInt8(2, +supported, 0);
  }
  static endSleepSupport(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createSleepSupport(builder, suspendNotifications, sleepInhibition, supported) {
    _SleepSupport.startSleepSupport(builder);
    _SleepSupport.addSuspendNotifications(builder, suspendNotifications);
    _SleepSupport.addSleepInhibition(builder, sleepInhibition);
    _SleepSupport.addSupported(builder, supported);
    return _SleepSupport.endSleepSupport(builder);
  }
  unpack() {
    return new SleepSupportT(
      this.suspendNotifications(),
      this.sleepInhibition(),
      this.supported()
    );
  }
  unpackTo(_o) {
    _o.suspendNotifications = this.suspendNotifications();
    _o.sleepInhibition = this.sleepInhibition();
    _o.supported = this.supported();
  }
};
var SleepSupportT = class {
  constructor(suspendNotifications = false, sleepInhibition = false, supported = false) {
    this.suspendNotifications = suspendNotifications;
    this.sleepInhibition = sleepInhibition;
    this.supported = supported;
  }
  pack(builder) {
    return SleepSupport.createSleepSupport(
      builder,
      this.suspendNotifications,
      this.sleepInhibition,
      this.supported
    );
  }
};

// generated/flatbuffers/revault/internal/stream-chunk.ts
var flatbuffers41 = __toESM(require_flatbuffers(), 1);
var StreamChunk = class _StreamChunk {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsStreamChunk(bb, obj) {
    return (obj || new _StreamChunk()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsStreamChunk(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers41.SIZE_PREFIX_LENGTH);
    return (obj || new _StreamChunk()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  path(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  fileOffset() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  length() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  physicalOffset() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  sparse() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  data(index) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  dataLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  dataArray() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  static startStreamChunk(builder) {
    builder.startObject(6);
  }
  static addPath(builder, pathOffset) {
    builder.addFieldOffset(0, pathOffset, 0);
  }
  static addFileOffset(builder, fileOffset) {
    builder.addFieldInt64(1, fileOffset, BigInt("0"));
  }
  static addLength(builder, length) {
    builder.addFieldInt64(2, length, BigInt("0"));
  }
  static addPhysicalOffset(builder, physicalOffset) {
    builder.addFieldInt64(3, physicalOffset, BigInt("0"));
  }
  static addSparse(builder, sparse) {
    builder.addFieldInt8(4, +sparse, 0);
  }
  static addData(builder, dataOffset) {
    builder.addFieldOffset(5, dataOffset, 0);
  }
  static createDataVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startDataVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static endStreamChunk(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createStreamChunk(builder, pathOffset, fileOffset, length, physicalOffset, sparse, dataOffset) {
    _StreamChunk.startStreamChunk(builder);
    _StreamChunk.addPath(builder, pathOffset);
    _StreamChunk.addFileOffset(builder, fileOffset);
    _StreamChunk.addLength(builder, length);
    _StreamChunk.addPhysicalOffset(builder, physicalOffset);
    _StreamChunk.addSparse(builder, sparse);
    _StreamChunk.addData(builder, dataOffset);
    return _StreamChunk.endStreamChunk(builder);
  }
  unpack() {
    return new StreamChunkT(
      this.path(),
      this.fileOffset(),
      this.length(),
      this.physicalOffset(),
      this.sparse(),
      this.bb.createScalarList(this.data.bind(this), this.dataLength())
    );
  }
  unpackTo(_o) {
    _o.path = this.path();
    _o.fileOffset = this.fileOffset();
    _o.length = this.length();
    _o.physicalOffset = this.physicalOffset();
    _o.sparse = this.sparse();
    _o.data = this.bb.createScalarList(this.data.bind(this), this.dataLength());
  }
};
var StreamChunkT = class {
  constructor(path = null, fileOffset = BigInt("0"), length = BigInt("0"), physicalOffset = BigInt("0"), sparse = false, data = []) {
    this.path = path;
    this.fileOffset = fileOffset;
    this.length = length;
    this.physicalOffset = physicalOffset;
    this.sparse = sparse;
    this.data = data;
  }
  pack(builder) {
    const path = this.path !== null ? builder.createString(this.path) : 0;
    const data = StreamChunk.createDataVector(builder, this.data);
    return StreamChunk.createStreamChunk(
      builder,
      path,
      this.fileOffset,
      this.length,
      this.physicalOffset,
      this.sparse,
      data
    );
  }
};

// generated/flatbuffers/revault/internal/stream-chunk-list.ts
var flatbuffers42 = __toESM(require_flatbuffers(), 1);
var StreamChunkList = class _StreamChunkList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsStreamChunkList(bb, obj) {
    return (obj || new _StreamChunkList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsStreamChunkList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers42.SIZE_PREFIX_LENGTH);
    return (obj || new _StreamChunkList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new StreamChunk()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startStreamChunkList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endStreamChunkList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createStreamChunkList(builder, valuesOffset) {
    _StreamChunkList.startStreamChunkList(builder);
    _StreamChunkList.addValues(builder, valuesOffset);
    return _StreamChunkList.endStreamChunkList(builder);
  }
  unpack() {
    return new StreamChunkListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var StreamChunkListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = StreamChunkList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return StreamChunkList.createStreamChunkList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/string-list.ts
var flatbuffers43 = __toESM(require_flatbuffers(), 1);
var StringList = class _StringList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsStringList(bb, obj) {
    return (obj || new _StringList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsStringList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers43.SIZE_PREFIX_LENGTH);
    return (obj || new _StringList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startStringList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endStringList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createStringList(builder, valuesOffset) {
    _StringList.startStringList(builder);
    _StringList.addValues(builder, valuesOffset);
    return _StringList.endStringList(builder);
  }
  unpack() {
    return new StringListT(
      this.bb.createScalarList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createScalarList(this.values.bind(this), this.valuesLength());
  }
};
var StringListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = StringList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return StringList.createStringList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/string-value.ts
var flatbuffers44 = __toESM(require_flatbuffers(), 1);
var StringValue = class _StringValue {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsStringValue(bb, obj) {
    return (obj || new _StringValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsStringValue(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers44.SIZE_PREFIX_LENGTH);
    return (obj || new _StringValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  value(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startStringValue(builder) {
    builder.startObject(1);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(0, valueOffset, 0);
  }
  static endStringValue(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createStringValue(builder, valueOffset) {
    _StringValue.startStringValue(builder);
    _StringValue.addValue(builder, valueOffset);
    return _StringValue.endStringValue(builder);
  }
  unpack() {
    return new StringValueT(
      this.value()
    );
  }
  unpackTo(_o) {
    _o.value = this.value();
  }
};
var StringValueT = class {
  constructor(value = null) {
    this.value = value;
  }
  pack(builder) {
    const value = this.value !== null ? builder.createString(this.value) : 0;
    return StringValue.createStringValue(
      builder,
      value
    );
  }
};

// generated/flatbuffers/revault/internal/variable.ts
var flatbuffers45 = __toESM(require_flatbuffers(), 1);
var Variable = class _Variable {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsVariable(bb, obj) {
    return (obj || new _Variable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsVariable(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers45.SIZE_PREFIX_LENGTH);
    return (obj || new _Variable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  sensitivity(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startVariable(builder) {
    builder.startObject(2);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addSensitivity(builder, sensitivityOffset) {
    builder.addFieldOffset(1, sensitivityOffset, 0);
  }
  static endVariable(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createVariable(builder, nameOffset, sensitivityOffset) {
    _Variable.startVariable(builder);
    _Variable.addName(builder, nameOffset);
    _Variable.addSensitivity(builder, sensitivityOffset);
    return _Variable.endVariable(builder);
  }
  unpack() {
    return new VariableT(
      this.name(),
      this.sensitivity()
    );
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.sensitivity = this.sensitivity();
  }
};
var VariableT = class {
  constructor(name = null, sensitivity = null) {
    this.name = name;
    this.sensitivity = sensitivity;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const sensitivity = this.sensitivity !== null ? builder.createString(this.sensitivity) : 0;
    return Variable.createVariable(
      builder,
      name,
      sensitivity
    );
  }
};

// generated/flatbuffers/revault/internal/variable-list.ts
var flatbuffers46 = __toESM(require_flatbuffers(), 1);
var VariableList = class _VariableList {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsVariableList(bb, obj) {
    return (obj || new _VariableList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsVariableList(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers46.SIZE_PREFIX_LENGTH);
    return (obj || new _VariableList()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new Variable()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static startVariableList(builder) {
    builder.startObject(1);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(0, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endVariableList(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createVariableList(builder, valuesOffset) {
    _VariableList.startVariableList(builder);
    _VariableList.addValues(builder, valuesOffset);
    return _VariableList.endVariableList(builder);
  }
  unpack() {
    return new VariableListT(
      this.bb.createObjList(this.values.bind(this), this.valuesLength())
    );
  }
  unpackTo(_o) {
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
  }
};
var VariableListT = class {
  constructor(values = []) {
    this.values = values;
  }
  pack(builder) {
    const values = VariableList.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    return VariableList.createVariableList(
      builder,
      values
    );
  }
};

// generated/flatbuffers/revault/internal/vault-backup-manifest.ts
var flatbuffers47 = __toESM(require_flatbuffers(), 1);
var VaultBackupManifest = class _VaultBackupManifest {
  bb = null;
  bb_pos = 0;
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsVaultBackupManifest(bb, obj) {
    return (obj || new _VaultBackupManifest()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsVaultBackupManifest(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers47.SIZE_PREFIX_LENGTH);
    return (obj || new _VaultBackupManifest()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  formatVersion() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  createdAtUnixMs() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  vaultFileName(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  vaultSize() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  vaultSha256(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static startVaultBackupManifest(builder) {
    builder.startObject(5);
  }
  static addFormatVersion(builder, formatVersion) {
    builder.addFieldInt32(0, formatVersion, 0);
  }
  static addCreatedAtUnixMs(builder, createdAtUnixMs) {
    builder.addFieldInt64(1, createdAtUnixMs, BigInt("0"));
  }
  static addVaultFileName(builder, vaultFileNameOffset) {
    builder.addFieldOffset(2, vaultFileNameOffset, 0);
  }
  static addVaultSize(builder, vaultSize) {
    builder.addFieldInt64(3, vaultSize, BigInt("0"));
  }
  static addVaultSha256(builder, vaultSha256Offset) {
    builder.addFieldOffset(4, vaultSha256Offset, 0);
  }
  static endVaultBackupManifest(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createVaultBackupManifest(builder, formatVersion, createdAtUnixMs, vaultFileNameOffset, vaultSize, vaultSha256Offset) {
    _VaultBackupManifest.startVaultBackupManifest(builder);
    _VaultBackupManifest.addFormatVersion(builder, formatVersion);
    _VaultBackupManifest.addCreatedAtUnixMs(builder, createdAtUnixMs);
    _VaultBackupManifest.addVaultFileName(builder, vaultFileNameOffset);
    _VaultBackupManifest.addVaultSize(builder, vaultSize);
    _VaultBackupManifest.addVaultSha256(builder, vaultSha256Offset);
    return _VaultBackupManifest.endVaultBackupManifest(builder);
  }
  unpack() {
    return new VaultBackupManifestT(
      this.formatVersion(),
      this.createdAtUnixMs(),
      this.vaultFileName(),
      this.vaultSize(),
      this.vaultSha256()
    );
  }
  unpackTo(_o) {
    _o.formatVersion = this.formatVersion();
    _o.createdAtUnixMs = this.createdAtUnixMs();
    _o.vaultFileName = this.vaultFileName();
    _o.vaultSize = this.vaultSize();
    _o.vaultSha256 = this.vaultSha256();
  }
};
var VaultBackupManifestT = class {
  constructor(formatVersion = 0, createdAtUnixMs = BigInt("0"), vaultFileName = null, vaultSize = BigInt("0"), vaultSha256 = null) {
    this.formatVersion = formatVersion;
    this.createdAtUnixMs = createdAtUnixMs;
    this.vaultFileName = vaultFileName;
    this.vaultSize = vaultSize;
    this.vaultSha256 = vaultSha256;
  }
  pack(builder) {
    const vaultFileName = this.vaultFileName !== null ? builder.createString(this.vaultFileName) : 0;
    const vaultSha256 = this.vaultSha256 !== null ? builder.createString(this.vaultSha256) : 0;
    return VaultBackupManifest.createVaultBackupManifest(
      builder,
      this.formatVersion,
      this.createdAtUnixMs,
      vaultFileName,
      this.vaultSize,
      vaultSha256
    );
  }
};
export {
  AccessSlotLabel,
  AccessSlotLabelList,
  AccessSlotLabelListT,
  AccessSlotLabelT,
  AgentEntry,
  AgentEntryList,
  AgentEntryListT,
  AgentEntryT,
  CacheStats,
  CacheStatsT,
  Contact,
  ContactList,
  ContactListT,
  ContactT,
  ErrorDetails,
  ErrorDetailsT,
  FileInspection,
  FileInspectionT,
  FormDefinition,
  FormDefinitionList,
  FormDefinitionListT,
  FormDefinitionT,
  FormField,
  FormFieldList,
  FormFieldListT,
  FormFieldT,
  FormRecord,
  FormRecordList,
  FormRecordListT,
  FormRecordT,
  FormValue,
  FormValueT,
  ImportStats,
  ImportStatsT,
  KeySlot,
  KeySlotList,
  KeySlotListT,
  KeySlotT,
  KnownLockbox,
  KnownLockboxList,
  KnownLockboxListT,
  KnownLockboxT,
  LockboxEntry,
  LockboxEntryKind,
  LockboxEntryList,
  LockboxEntryListT,
  LockboxEntryT,
  OptionalFormRecord,
  OptionalFormRecordT,
  OptionalFormValue,
  OptionalFormValueT,
  OptionalLockboxEntry,
  OptionalLockboxEntryT,
  OptionalString,
  OptionalStringT,
  OwnerInspection,
  OwnerInspectionT,
  PageInspection,
  PageInspectionList,
  PageInspectionListT,
  PageInspectionT,
  PageObject,
  PageObjectT,
  PathMove,
  PathMoveList,
  PathMoveListT,
  PathMoveT,
  PlatformStatus,
  PlatformStatusT,
  ProfileGeneration,
  ProfileGenerationT,
  ProfileHistory,
  ProfileHistoryList,
  ProfileHistoryListT,
  ProfileHistoryT,
  RecoveryReport,
  RecoveryReportT,
  RuntimeOptions,
  RuntimeOptionsT,
  SleepSupport,
  SleepSupportT,
  StreamChunk,
  StreamChunkList,
  StreamChunkListT,
  StreamChunkT,
  StringList,
  StringListT,
  StringValue,
  StringValueT,
  Variable,
  VariableList,
  VariableListT,
  VariableT,
  VaultBackupManifest,
  VaultBackupManifestT
};
