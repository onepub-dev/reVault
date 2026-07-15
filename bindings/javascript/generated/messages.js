/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
import protobufModule from "protobufjs/minimal.js";

// protobufjs is CommonJS today, but Node's ESM interop shape differs depending
// on whether this package is installed directly or through another workspace.
const $protobuf = protobufModule.default ?? protobufModule;

// Common aliases
const $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
const $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

export const revault = $root.revault = (() => {

    /**
     * Namespace revault.
     * @exports revault
     * @namespace
     */
    const revault = {};

    revault.bindings = (function() {

        /**
         * Namespace bindings.
         * @memberof revault
         * @namespace
         */
        const bindings = {};

        bindings.LockboxEntry = (function() {

            /**
             * Properties of a LockboxEntry.
             * @memberof revault.bindings
             * @interface ILockboxEntry
             * @property {string|null} [path] LockboxEntry path
             * @property {revault.bindings.LockboxEntry.Kind|null} [kind] LockboxEntry kind
             * @property {number|Long|null} [length] LockboxEntry length
             * @property {number|null} [permissions] LockboxEntry permissions
             */

            /**
             * Constructs a new LockboxEntry.
             * @memberof revault.bindings
             * @classdesc Represents a LockboxEntry.
             * @implements ILockboxEntry
             * @constructor
             * @param {revault.bindings.ILockboxEntry=} [properties] Properties to set
             */
            function LockboxEntry(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * LockboxEntry path.
             * @member {string} path
             * @memberof revault.bindings.LockboxEntry
             * @instance
             */
            LockboxEntry.prototype.path = "";

            /**
             * LockboxEntry kind.
             * @member {revault.bindings.LockboxEntry.Kind} kind
             * @memberof revault.bindings.LockboxEntry
             * @instance
             */
            LockboxEntry.prototype.kind = 0;

            /**
             * LockboxEntry length.
             * @member {number|Long} length
             * @memberof revault.bindings.LockboxEntry
             * @instance
             */
            LockboxEntry.prototype.length = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * LockboxEntry permissions.
             * @member {number} permissions
             * @memberof revault.bindings.LockboxEntry
             * @instance
             */
            LockboxEntry.prototype.permissions = 0;

            /**
             * Creates a new LockboxEntry instance using the specified properties.
             * @function create
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {revault.bindings.ILockboxEntry=} [properties] Properties to set
             * @returns {revault.bindings.LockboxEntry} LockboxEntry instance
             */
            LockboxEntry.create = function create(properties) {
                return new LockboxEntry(properties);
            };

            /**
             * Encodes the specified LockboxEntry message. Does not implicitly {@link revault.bindings.LockboxEntry.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {revault.bindings.ILockboxEntry} message LockboxEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LockboxEntry.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.path);
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    writer.uint32(/* id 2, wireType 0 =*/16).int32(message.kind);
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.length);
                if (message.permissions != null && Object.hasOwnProperty.call(message, "permissions"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.permissions);
                return writer;
            };

            /**
             * Encodes the specified LockboxEntry message, length delimited. Does not implicitly {@link revault.bindings.LockboxEntry.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {revault.bindings.ILockboxEntry} message LockboxEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LockboxEntry.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a LockboxEntry message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.LockboxEntry} LockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LockboxEntry.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.LockboxEntry();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.path = reader.string();
                            break;
                        }
                    case 2: {
                            message.kind = reader.int32();
                            break;
                        }
                    case 3: {
                            message.length = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.permissions = reader.uint32();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a LockboxEntry message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.LockboxEntry} LockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LockboxEntry.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a LockboxEntry message.
             * @function verify
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            LockboxEntry.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    if (!$util.isString(message.path))
                        return "path: string expected";
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    switch (message.kind) {
                    default:
                        return "kind: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                    case 3:
                        break;
                    }
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    if (!$util.isInteger(message.length) && !(message.length && $util.isInteger(message.length.low) && $util.isInteger(message.length.high)))
                        return "length: integer|Long expected";
                if (message.permissions != null && Object.hasOwnProperty.call(message, "permissions"))
                    if (!$util.isInteger(message.permissions))
                        return "permissions: integer expected";
                return null;
            };

            /**
             * Creates a LockboxEntry message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.LockboxEntry} LockboxEntry
             */
            LockboxEntry.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.LockboxEntry)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.LockboxEntry: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.LockboxEntry();
                if (object.path != null)
                    message.path = String(object.path);
                switch (object.kind) {
                default:
                    if (typeof object.kind === "number") {
                        message.kind = object.kind;
                        break;
                    }
                    break;
                case "KIND_UNSPECIFIED":
                case 0:
                    message.kind = 0;
                    break;
                case "FILE":
                case 1:
                    message.kind = 1;
                    break;
                case "SYMLINK":
                case 2:
                    message.kind = 2;
                    break;
                case "DIRECTORY":
                case 3:
                    message.kind = 3;
                    break;
                }
                if (object.length != null)
                    if ($util.Long)
                        message.length = $util.Long.fromValue(object.length, true);
                    else if (typeof object.length === "string")
                        message.length = parseInt(object.length, 10);
                    else if (typeof object.length === "number")
                        message.length = object.length;
                    else if (typeof object.length === "object")
                        message.length = new $util.LongBits(object.length.low >>> 0, object.length.high >>> 0).toNumber(true);
                if (object.permissions != null)
                    message.permissions = object.permissions >>> 0;
                return message;
            };

            /**
             * Creates a plain object from a LockboxEntry message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {revault.bindings.LockboxEntry} message LockboxEntry
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            LockboxEntry.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.path = "";
                    object.kind = options.enums === String ? "KIND_UNSPECIFIED" : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.length = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.length = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.permissions = 0;
                }
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    object.path = message.path;
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    object.kind = options.enums === String ? $root.revault.bindings.LockboxEntry.Kind[message.kind] === undefined ? message.kind : $root.revault.bindings.LockboxEntry.Kind[message.kind] : message.kind;
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.length = typeof message.length === "number" ? BigInt(message.length) : $util.Long.fromBits(message.length.low >>> 0, message.length.high >>> 0, true).toBigInt();
                    else if (typeof message.length === "number")
                        object.length = options.longs === String ? String(message.length) : message.length;
                    else
                        object.length = options.longs === String ? $util.Long.prototype.toString.call(message.length) : options.longs === Number ? new $util.LongBits(message.length.low >>> 0, message.length.high >>> 0).toNumber(true) : message.length;
                if (message.permissions != null && Object.hasOwnProperty.call(message, "permissions"))
                    object.permissions = message.permissions;
                return object;
            };

            /**
             * Converts this LockboxEntry to JSON.
             * @function toJSON
             * @memberof revault.bindings.LockboxEntry
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            LockboxEntry.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for LockboxEntry
             * @function getTypeUrl
             * @memberof revault.bindings.LockboxEntry
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            LockboxEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.LockboxEntry";
            };

            /**
             * Kind enum.
             * @name revault.bindings.LockboxEntry.Kind
             * @enum {number}
             * @property {number} KIND_UNSPECIFIED=0 KIND_UNSPECIFIED value
             * @property {number} FILE=1 FILE value
             * @property {number} SYMLINK=2 SYMLINK value
             * @property {number} DIRECTORY=3 DIRECTORY value
             */
            LockboxEntry.Kind = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "KIND_UNSPECIFIED"] = 0;
                values[valuesById[1] = "FILE"] = 1;
                values[valuesById[2] = "SYMLINK"] = 2;
                values[valuesById[3] = "DIRECTORY"] = 3;
                return values;
            })();

            return LockboxEntry;
        })();

        bindings.LockboxEntryList = (function() {

            /**
             * Properties of a LockboxEntryList.
             * @memberof revault.bindings
             * @interface ILockboxEntryList
             * @property {Array.<revault.bindings.ILockboxEntry>|null} [entries] LockboxEntryList entries
             */

            /**
             * Constructs a new LockboxEntryList.
             * @memberof revault.bindings
             * @classdesc Represents a LockboxEntryList.
             * @implements ILockboxEntryList
             * @constructor
             * @param {revault.bindings.ILockboxEntryList=} [properties] Properties to set
             */
            function LockboxEntryList(properties) {
                this.entries = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * LockboxEntryList entries.
             * @member {Array.<revault.bindings.ILockboxEntry>} entries
             * @memberof revault.bindings.LockboxEntryList
             * @instance
             */
            LockboxEntryList.prototype.entries = $util.emptyArray;

            /**
             * Creates a new LockboxEntryList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {revault.bindings.ILockboxEntryList=} [properties] Properties to set
             * @returns {revault.bindings.LockboxEntryList} LockboxEntryList instance
             */
            LockboxEntryList.create = function create(properties) {
                return new LockboxEntryList(properties);
            };

            /**
             * Encodes the specified LockboxEntryList message. Does not implicitly {@link revault.bindings.LockboxEntryList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {revault.bindings.ILockboxEntryList} message LockboxEntryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LockboxEntryList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.entries != null && message.entries.length)
                    for (let i = 0; i < message.entries.length; ++i)
                        $root.revault.bindings.LockboxEntry.encode(message.entries[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified LockboxEntryList message, length delimited. Does not implicitly {@link revault.bindings.LockboxEntryList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {revault.bindings.ILockboxEntryList} message LockboxEntryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LockboxEntryList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a LockboxEntryList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.LockboxEntryList} LockboxEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LockboxEntryList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.LockboxEntryList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.entries && message.entries.length))
                                message.entries = [];
                            message.entries.push($root.revault.bindings.LockboxEntry.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a LockboxEntryList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.LockboxEntryList} LockboxEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LockboxEntryList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a LockboxEntryList message.
             * @function verify
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            LockboxEntryList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.entries != null && Object.hasOwnProperty.call(message, "entries")) {
                    if (!Array.isArray(message.entries))
                        return "entries: array expected";
                    for (let i = 0; i < message.entries.length; ++i) {
                        let error = $root.revault.bindings.LockboxEntry.verify(message.entries[i], long + 1);
                        if (error)
                            return "entries." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a LockboxEntryList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.LockboxEntryList} LockboxEntryList
             */
            LockboxEntryList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.LockboxEntryList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.LockboxEntryList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.LockboxEntryList();
                if (object.entries) {
                    if (!Array.isArray(object.entries))
                        throw TypeError(".revault.bindings.LockboxEntryList.entries: array expected");
                    message.entries = [];
                    for (let i = 0; i < object.entries.length; ++i) {
                        if (!$util.isObject(object.entries[i]))
                            throw TypeError(".revault.bindings.LockboxEntryList.entries: object expected");
                        message.entries[i] = $root.revault.bindings.LockboxEntry.fromObject(object.entries[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a LockboxEntryList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {revault.bindings.LockboxEntryList} message LockboxEntryList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            LockboxEntryList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.entries = [];
                if (message.entries && message.entries.length) {
                    object.entries = [];
                    for (let j = 0; j < message.entries.length; ++j)
                        object.entries[j] = $root.revault.bindings.LockboxEntry.toObject(message.entries[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this LockboxEntryList to JSON.
             * @function toJSON
             * @memberof revault.bindings.LockboxEntryList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            LockboxEntryList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for LockboxEntryList
             * @function getTypeUrl
             * @memberof revault.bindings.LockboxEntryList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            LockboxEntryList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.LockboxEntryList";
            };

            return LockboxEntryList;
        })();

        bindings.OptionalLockboxEntry = (function() {

            /**
             * Properties of an OptionalLockboxEntry.
             * @memberof revault.bindings
             * @interface IOptionalLockboxEntry
             * @property {revault.bindings.ILockboxEntry|null} [value] OptionalLockboxEntry value
             */

            /**
             * Constructs a new OptionalLockboxEntry.
             * @memberof revault.bindings
             * @classdesc Represents an OptionalLockboxEntry.
             * @implements IOptionalLockboxEntry
             * @constructor
             * @param {revault.bindings.IOptionalLockboxEntry=} [properties] Properties to set
             */
            function OptionalLockboxEntry(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OptionalLockboxEntry value.
             * @member {revault.bindings.ILockboxEntry|null|undefined} value
             * @memberof revault.bindings.OptionalLockboxEntry
             * @instance
             */
            OptionalLockboxEntry.prototype.value = null;

            /**
             * Creates a new OptionalLockboxEntry instance using the specified properties.
             * @function create
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {revault.bindings.IOptionalLockboxEntry=} [properties] Properties to set
             * @returns {revault.bindings.OptionalLockboxEntry} OptionalLockboxEntry instance
             */
            OptionalLockboxEntry.create = function create(properties) {
                return new OptionalLockboxEntry(properties);
            };

            /**
             * Encodes the specified OptionalLockboxEntry message. Does not implicitly {@link revault.bindings.OptionalLockboxEntry.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {revault.bindings.IOptionalLockboxEntry} message OptionalLockboxEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalLockboxEntry.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    $root.revault.bindings.LockboxEntry.encode(message.value, writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified OptionalLockboxEntry message, length delimited. Does not implicitly {@link revault.bindings.OptionalLockboxEntry.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {revault.bindings.IOptionalLockboxEntry} message OptionalLockboxEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalLockboxEntry.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OptionalLockboxEntry message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.OptionalLockboxEntry} OptionalLockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalLockboxEntry.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.OptionalLockboxEntry();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.value = $root.revault.bindings.LockboxEntry.decode(reader, reader.uint32(), undefined, long + 1);
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OptionalLockboxEntry message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.OptionalLockboxEntry} OptionalLockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalLockboxEntry.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OptionalLockboxEntry message.
             * @function verify
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OptionalLockboxEntry.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.value != null && Object.hasOwnProperty.call(message, "value")) {
                    let error = $root.revault.bindings.LockboxEntry.verify(message.value, long + 1);
                    if (error)
                        return "value." + error;
                }
                return null;
            };

            /**
             * Creates an OptionalLockboxEntry message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.OptionalLockboxEntry} OptionalLockboxEntry
             */
            OptionalLockboxEntry.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.OptionalLockboxEntry)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.OptionalLockboxEntry: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.OptionalLockboxEntry();
                if (object.value != null) {
                    if (!$util.isObject(object.value))
                        throw TypeError(".revault.bindings.OptionalLockboxEntry.value: object expected");
                    message.value = $root.revault.bindings.LockboxEntry.fromObject(object.value, long + 1);
                }
                return message;
            };

            /**
             * Creates a plain object from an OptionalLockboxEntry message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {revault.bindings.OptionalLockboxEntry} message OptionalLockboxEntry
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OptionalLockboxEntry.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults)
                    object.value = null;
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = $root.revault.bindings.LockboxEntry.toObject(message.value, options, q + 1);
                return object;
            };

            /**
             * Converts this OptionalLockboxEntry to JSON.
             * @function toJSON
             * @memberof revault.bindings.OptionalLockboxEntry
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OptionalLockboxEntry.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for OptionalLockboxEntry
             * @function getTypeUrl
             * @memberof revault.bindings.OptionalLockboxEntry
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            OptionalLockboxEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.OptionalLockboxEntry";
            };

            return OptionalLockboxEntry;
        })();

        bindings.StringList = (function() {

            /**
             * Properties of a StringList.
             * @memberof revault.bindings
             * @interface IStringList
             * @property {Array.<string>|null} [values] StringList values
             */

            /**
             * Constructs a new StringList.
             * @memberof revault.bindings
             * @classdesc Represents a StringList.
             * @implements IStringList
             * @constructor
             * @param {revault.bindings.IStringList=} [properties] Properties to set
             */
            function StringList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * StringList values.
             * @member {Array.<string>} values
             * @memberof revault.bindings.StringList
             * @instance
             */
            StringList.prototype.values = $util.emptyArray;

            /**
             * Creates a new StringList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.StringList
             * @static
             * @param {revault.bindings.IStringList=} [properties] Properties to set
             * @returns {revault.bindings.StringList} StringList instance
             */
            StringList.create = function create(properties) {
                return new StringList(properties);
            };

            /**
             * Encodes the specified StringList message. Does not implicitly {@link revault.bindings.StringList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.StringList
             * @static
             * @param {revault.bindings.IStringList} message StringList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StringList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.values[i]);
                return writer;
            };

            /**
             * Encodes the specified StringList message, length delimited. Does not implicitly {@link revault.bindings.StringList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.StringList
             * @static
             * @param {revault.bindings.IStringList} message StringList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StringList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a StringList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.StringList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.StringList} StringList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StringList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.StringList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push(reader.string());
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a StringList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.StringList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.StringList} StringList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StringList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a StringList message.
             * @function verify
             * @memberof revault.bindings.StringList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StringList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i)
                        if (!$util.isString(message.values[i]))
                            return "values: string[] expected";
                }
                return null;
            };

            /**
             * Creates a StringList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.StringList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.StringList} StringList
             */
            StringList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.StringList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.StringList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.StringList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.StringList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i)
                        message.values[i] = String(object.values[i]);
                }
                return message;
            };

            /**
             * Creates a plain object from a StringList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.StringList
             * @static
             * @param {revault.bindings.StringList} message StringList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StringList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = message.values[j];
                }
                return object;
            };

            /**
             * Converts this StringList to JSON.
             * @function toJSON
             * @memberof revault.bindings.StringList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StringList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for StringList
             * @function getTypeUrl
             * @memberof revault.bindings.StringList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            StringList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.StringList";
            };

            return StringList;
        })();

        bindings.PathMove = (function() {

            /**
             * Properties of a PathMove.
             * @memberof revault.bindings
             * @interface IPathMove
             * @property {string|null} [source] PathMove source
             * @property {string|null} [destination] PathMove destination
             */

            /**
             * Constructs a new PathMove.
             * @memberof revault.bindings
             * @classdesc Represents a PathMove.
             * @implements IPathMove
             * @constructor
             * @param {revault.bindings.IPathMove=} [properties] Properties to set
             */
            function PathMove(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PathMove source.
             * @member {string} source
             * @memberof revault.bindings.PathMove
             * @instance
             */
            PathMove.prototype.source = "";

            /**
             * PathMove destination.
             * @member {string} destination
             * @memberof revault.bindings.PathMove
             * @instance
             */
            PathMove.prototype.destination = "";

            /**
             * Creates a new PathMove instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PathMove
             * @static
             * @param {revault.bindings.IPathMove=} [properties] Properties to set
             * @returns {revault.bindings.PathMove} PathMove instance
             */
            PathMove.create = function create(properties) {
                return new PathMove(properties);
            };

            /**
             * Encodes the specified PathMove message. Does not implicitly {@link revault.bindings.PathMove.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PathMove
             * @static
             * @param {revault.bindings.IPathMove} message PathMove message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PathMove.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.source != null && Object.hasOwnProperty.call(message, "source"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.source);
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.destination);
                return writer;
            };

            /**
             * Encodes the specified PathMove message, length delimited. Does not implicitly {@link revault.bindings.PathMove.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PathMove
             * @static
             * @param {revault.bindings.IPathMove} message PathMove message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PathMove.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PathMove message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PathMove
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PathMove} PathMove
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PathMove.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PathMove();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.source = reader.string();
                            break;
                        }
                    case 2: {
                            message.destination = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PathMove message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PathMove
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PathMove} PathMove
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PathMove.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PathMove message.
             * @function verify
             * @memberof revault.bindings.PathMove
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PathMove.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.source != null && Object.hasOwnProperty.call(message, "source"))
                    if (!$util.isString(message.source))
                        return "source: string expected";
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    if (!$util.isString(message.destination))
                        return "destination: string expected";
                return null;
            };

            /**
             * Creates a PathMove message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PathMove
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PathMove} PathMove
             */
            PathMove.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PathMove)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PathMove: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PathMove();
                if (object.source != null)
                    message.source = String(object.source);
                if (object.destination != null)
                    message.destination = String(object.destination);
                return message;
            };

            /**
             * Creates a plain object from a PathMove message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PathMove
             * @static
             * @param {revault.bindings.PathMove} message PathMove
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PathMove.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.source = "";
                    object.destination = "";
                }
                if (message.source != null && Object.hasOwnProperty.call(message, "source"))
                    object.source = message.source;
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    object.destination = message.destination;
                return object;
            };

            /**
             * Converts this PathMove to JSON.
             * @function toJSON
             * @memberof revault.bindings.PathMove
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PathMove.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PathMove
             * @function getTypeUrl
             * @memberof revault.bindings.PathMove
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PathMove.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PathMove";
            };

            return PathMove;
        })();

        bindings.PathMoveList = (function() {

            /**
             * Properties of a PathMoveList.
             * @memberof revault.bindings
             * @interface IPathMoveList
             * @property {Array.<revault.bindings.IPathMove>|null} [values] PathMoveList values
             */

            /**
             * Constructs a new PathMoveList.
             * @memberof revault.bindings
             * @classdesc Represents a PathMoveList.
             * @implements IPathMoveList
             * @constructor
             * @param {revault.bindings.IPathMoveList=} [properties] Properties to set
             */
            function PathMoveList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PathMoveList values.
             * @member {Array.<revault.bindings.IPathMove>} values
             * @memberof revault.bindings.PathMoveList
             * @instance
             */
            PathMoveList.prototype.values = $util.emptyArray;

            /**
             * Creates a new PathMoveList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {revault.bindings.IPathMoveList=} [properties] Properties to set
             * @returns {revault.bindings.PathMoveList} PathMoveList instance
             */
            PathMoveList.create = function create(properties) {
                return new PathMoveList(properties);
            };

            /**
             * Encodes the specified PathMoveList message. Does not implicitly {@link revault.bindings.PathMoveList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {revault.bindings.IPathMoveList} message PathMoveList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PathMoveList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.PathMove.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified PathMoveList message, length delimited. Does not implicitly {@link revault.bindings.PathMoveList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {revault.bindings.IPathMoveList} message PathMoveList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PathMoveList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PathMoveList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PathMoveList} PathMoveList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PathMoveList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PathMoveList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.PathMove.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PathMoveList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PathMoveList} PathMoveList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PathMoveList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PathMoveList message.
             * @function verify
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PathMoveList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.PathMove.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a PathMoveList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PathMoveList} PathMoveList
             */
            PathMoveList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PathMoveList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PathMoveList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PathMoveList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.PathMoveList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.PathMoveList.values: object expected");
                        message.values[i] = $root.revault.bindings.PathMove.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a PathMoveList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {revault.bindings.PathMoveList} message PathMoveList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PathMoveList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.PathMove.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this PathMoveList to JSON.
             * @function toJSON
             * @memberof revault.bindings.PathMoveList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PathMoveList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PathMoveList
             * @function getTypeUrl
             * @memberof revault.bindings.PathMoveList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PathMoveList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PathMoveList";
            };

            return PathMoveList;
        })();

        bindings.ByteList = (function() {

            /**
             * Properties of a ByteList.
             * @memberof revault.bindings
             * @interface IByteList
             * @property {Array.<Uint8Array>|null} [values] ByteList values
             */

            /**
             * Constructs a new ByteList.
             * @memberof revault.bindings
             * @classdesc Represents a ByteList.
             * @implements IByteList
             * @constructor
             * @param {revault.bindings.IByteList=} [properties] Properties to set
             */
            function ByteList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ByteList values.
             * @member {Array.<Uint8Array>} values
             * @memberof revault.bindings.ByteList
             * @instance
             */
            ByteList.prototype.values = $util.emptyArray;

            /**
             * Creates a new ByteList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ByteList
             * @static
             * @param {revault.bindings.IByteList=} [properties] Properties to set
             * @returns {revault.bindings.ByteList} ByteList instance
             */
            ByteList.create = function create(properties) {
                return new ByteList(properties);
            };

            /**
             * Encodes the specified ByteList message. Does not implicitly {@link revault.bindings.ByteList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ByteList
             * @static
             * @param {revault.bindings.IByteList} message ByteList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ByteList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.values[i]);
                return writer;
            };

            /**
             * Encodes the specified ByteList message, length delimited. Does not implicitly {@link revault.bindings.ByteList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ByteList
             * @static
             * @param {revault.bindings.IByteList} message ByteList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ByteList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ByteList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ByteList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ByteList} ByteList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ByteList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ByteList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push(reader.bytes());
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ByteList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ByteList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ByteList} ByteList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ByteList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ByteList message.
             * @function verify
             * @memberof revault.bindings.ByteList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ByteList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i)
                        if (!(message.values[i] && typeof message.values[i].length === "number" || $util.isString(message.values[i])))
                            return "values: buffer[] expected";
                }
                return null;
            };

            /**
             * Creates a ByteList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ByteList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ByteList} ByteList
             */
            ByteList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ByteList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ByteList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ByteList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.ByteList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i)
                        if (typeof object.values[i] === "string")
                            $util.base64.decode(object.values[i], message.values[i] = $util.newBuffer($util.base64.length(object.values[i])), 0);
                        else if (object.values[i].length >= 0)
                            message.values[i] = object.values[i];
                }
                return message;
            };

            /**
             * Creates a plain object from a ByteList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ByteList
             * @static
             * @param {revault.bindings.ByteList} message ByteList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ByteList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = options.bytes === String ? $util.base64.encode(message.values[j], 0, message.values[j].length) : options.bytes === Array ? Array.prototype.slice.call(message.values[j]) : message.values[j];
                }
                return object;
            };

            /**
             * Converts this ByteList to JSON.
             * @function toJSON
             * @memberof revault.bindings.ByteList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ByteList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ByteList
             * @function getTypeUrl
             * @memberof revault.bindings.ByteList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ByteList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ByteList";
            };

            return ByteList;
        })();

        bindings.FormField = (function() {

            /**
             * Properties of a FormField.
             * @memberof revault.bindings
             * @interface IFormField
             * @property {string|null} [id] FormField id
             * @property {string|null} [label] FormField label
             * @property {string|null} [kind] FormField kind
             * @property {boolean|null} [required] FormField required
             */

            /**
             * Constructs a new FormField.
             * @memberof revault.bindings
             * @classdesc Represents a FormField.
             * @implements IFormField
             * @constructor
             * @param {revault.bindings.IFormField=} [properties] Properties to set
             */
            function FormField(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormField id.
             * @member {string} id
             * @memberof revault.bindings.FormField
             * @instance
             */
            FormField.prototype.id = "";

            /**
             * FormField label.
             * @member {string} label
             * @memberof revault.bindings.FormField
             * @instance
             */
            FormField.prototype.label = "";

            /**
             * FormField kind.
             * @member {string} kind
             * @memberof revault.bindings.FormField
             * @instance
             */
            FormField.prototype.kind = "";

            /**
             * FormField required.
             * @member {boolean} required
             * @memberof revault.bindings.FormField
             * @instance
             */
            FormField.prototype.required = false;

            /**
             * Creates a new FormField instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormField
             * @static
             * @param {revault.bindings.IFormField=} [properties] Properties to set
             * @returns {revault.bindings.FormField} FormField instance
             */
            FormField.create = function create(properties) {
                return new FormField(properties);
            };

            /**
             * Encodes the specified FormField message. Does not implicitly {@link revault.bindings.FormField.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormField
             * @static
             * @param {revault.bindings.IFormField} message FormField message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormField.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.label);
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.kind);
                if (message.required != null && Object.hasOwnProperty.call(message, "required"))
                    writer.uint32(/* id 4, wireType 0 =*/32).bool(message.required);
                return writer;
            };

            /**
             * Encodes the specified FormField message, length delimited. Does not implicitly {@link revault.bindings.FormField.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormField
             * @static
             * @param {revault.bindings.IFormField} message FormField message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormField.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormField message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormField
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormField} FormField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormField.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormField();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.id = reader.string();
                            break;
                        }
                    case 2: {
                            message.label = reader.string();
                            break;
                        }
                    case 3: {
                            message.kind = reader.string();
                            break;
                        }
                    case 4: {
                            message.required = reader.bool();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormField message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormField
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormField} FormField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormField.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormField message.
             * @function verify
             * @memberof revault.bindings.FormField
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormField.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (!$util.isString(message.id))
                        return "id: string expected";
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    if (!$util.isString(message.kind))
                        return "kind: string expected";
                if (message.required != null && Object.hasOwnProperty.call(message, "required"))
                    if (typeof message.required !== "boolean")
                        return "required: boolean expected";
                return null;
            };

            /**
             * Creates a FormField message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormField
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormField} FormField
             */
            FormField.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormField)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormField: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormField();
                if (object.id != null)
                    message.id = String(object.id);
                if (object.label != null)
                    message.label = String(object.label);
                if (object.kind != null)
                    message.kind = String(object.kind);
                if (object.required != null)
                    message.required = Boolean(object.required);
                return message;
            };

            /**
             * Creates a plain object from a FormField message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormField
             * @static
             * @param {revault.bindings.FormField} message FormField
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormField.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.id = "";
                    object.label = "";
                    object.kind = "";
                    object.required = false;
                }
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    object.id = message.id;
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    object.label = message.label;
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    object.kind = message.kind;
                if (message.required != null && Object.hasOwnProperty.call(message, "required"))
                    object.required = message.required;
                return object;
            };

            /**
             * Converts this FormField to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormField
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormField.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormField
             * @function getTypeUrl
             * @memberof revault.bindings.FormField
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormField.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormField";
            };

            return FormField;
        })();

        bindings.FormFieldList = (function() {

            /**
             * Properties of a FormFieldList.
             * @memberof revault.bindings
             * @interface IFormFieldList
             * @property {Array.<revault.bindings.IFormField>|null} [values] FormFieldList values
             */

            /**
             * Constructs a new FormFieldList.
             * @memberof revault.bindings
             * @classdesc Represents a FormFieldList.
             * @implements IFormFieldList
             * @constructor
             * @param {revault.bindings.IFormFieldList=} [properties] Properties to set
             */
            function FormFieldList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormFieldList values.
             * @member {Array.<revault.bindings.IFormField>} values
             * @memberof revault.bindings.FormFieldList
             * @instance
             */
            FormFieldList.prototype.values = $util.emptyArray;

            /**
             * Creates a new FormFieldList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {revault.bindings.IFormFieldList=} [properties] Properties to set
             * @returns {revault.bindings.FormFieldList} FormFieldList instance
             */
            FormFieldList.create = function create(properties) {
                return new FormFieldList(properties);
            };

            /**
             * Encodes the specified FormFieldList message. Does not implicitly {@link revault.bindings.FormFieldList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {revault.bindings.IFormFieldList} message FormFieldList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormFieldList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.FormField.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FormFieldList message, length delimited. Does not implicitly {@link revault.bindings.FormFieldList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {revault.bindings.IFormFieldList} message FormFieldList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormFieldList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormFieldList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormFieldList} FormFieldList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormFieldList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormFieldList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.FormField.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormFieldList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormFieldList} FormFieldList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormFieldList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormFieldList message.
             * @function verify
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormFieldList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.FormField.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FormFieldList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormFieldList} FormFieldList
             */
            FormFieldList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormFieldList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormFieldList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormFieldList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.FormFieldList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.FormFieldList.values: object expected");
                        message.values[i] = $root.revault.bindings.FormField.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FormFieldList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {revault.bindings.FormFieldList} message FormFieldList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormFieldList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.FormField.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FormFieldList to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormFieldList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormFieldList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormFieldList
             * @function getTypeUrl
             * @memberof revault.bindings.FormFieldList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormFieldList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormFieldList";
            };

            return FormFieldList;
        })();

        bindings.FormDefinition = (function() {

            /**
             * Properties of a FormDefinition.
             * @memberof revault.bindings
             * @interface IFormDefinition
             * @property {string|null} [typeId] FormDefinition typeId
             * @property {string|null} [alias] FormDefinition alias
             * @property {number|null} [revision] FormDefinition revision
             * @property {string|null} [name] FormDefinition name
             * @property {string|null} [description] FormDefinition description
             * @property {Array.<revault.bindings.IFormField>|null} [fields] FormDefinition fields
             */

            /**
             * Constructs a new FormDefinition.
             * @memberof revault.bindings
             * @classdesc Represents a FormDefinition.
             * @implements IFormDefinition
             * @constructor
             * @param {revault.bindings.IFormDefinition=} [properties] Properties to set
             */
            function FormDefinition(properties) {
                this.fields = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormDefinition typeId.
             * @member {string} typeId
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.typeId = "";

            /**
             * FormDefinition alias.
             * @member {string} alias
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.alias = "";

            /**
             * FormDefinition revision.
             * @member {number} revision
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.revision = 0;

            /**
             * FormDefinition name.
             * @member {string} name
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.name = "";

            /**
             * FormDefinition description.
             * @member {string} description
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.description = "";

            /**
             * FormDefinition fields.
             * @member {Array.<revault.bindings.IFormField>} fields
             * @memberof revault.bindings.FormDefinition
             * @instance
             */
            FormDefinition.prototype.fields = $util.emptyArray;

            /**
             * Creates a new FormDefinition instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {revault.bindings.IFormDefinition=} [properties] Properties to set
             * @returns {revault.bindings.FormDefinition} FormDefinition instance
             */
            FormDefinition.create = function create(properties) {
                return new FormDefinition(properties);
            };

            /**
             * Encodes the specified FormDefinition message. Does not implicitly {@link revault.bindings.FormDefinition.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {revault.bindings.IFormDefinition} message FormDefinition message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormDefinition.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.typeId);
                if (message.alias != null && Object.hasOwnProperty.call(message, "alias"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.alias);
                if (message.revision != null && Object.hasOwnProperty.call(message, "revision"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.revision);
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.name);
                if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.description);
                if (message.fields != null && message.fields.length)
                    for (let i = 0; i < message.fields.length; ++i)
                        $root.revault.bindings.FormField.encode(message.fields[i], writer.uint32(/* id 6, wireType 2 =*/50).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FormDefinition message, length delimited. Does not implicitly {@link revault.bindings.FormDefinition.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {revault.bindings.IFormDefinition} message FormDefinition message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormDefinition.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormDefinition message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormDefinition} FormDefinition
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormDefinition.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormDefinition();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.typeId = reader.string();
                            break;
                        }
                    case 2: {
                            message.alias = reader.string();
                            break;
                        }
                    case 3: {
                            message.revision = reader.uint32();
                            break;
                        }
                    case 4: {
                            message.name = reader.string();
                            break;
                        }
                    case 5: {
                            message.description = reader.string();
                            break;
                        }
                    case 6: {
                            if (!(message.fields && message.fields.length))
                                message.fields = [];
                            message.fields.push($root.revault.bindings.FormField.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormDefinition message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormDefinition} FormDefinition
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormDefinition.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormDefinition message.
             * @function verify
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormDefinition.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    if (!$util.isString(message.typeId))
                        return "typeId: string expected";
                if (message.alias != null && Object.hasOwnProperty.call(message, "alias"))
                    if (!$util.isString(message.alias))
                        return "alias: string expected";
                if (message.revision != null && Object.hasOwnProperty.call(message, "revision"))
                    if (!$util.isInteger(message.revision))
                        return "revision: integer expected";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                    if (!$util.isString(message.description))
                        return "description: string expected";
                if (message.fields != null && Object.hasOwnProperty.call(message, "fields")) {
                    if (!Array.isArray(message.fields))
                        return "fields: array expected";
                    for (let i = 0; i < message.fields.length; ++i) {
                        let error = $root.revault.bindings.FormField.verify(message.fields[i], long + 1);
                        if (error)
                            return "fields." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FormDefinition message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormDefinition} FormDefinition
             */
            FormDefinition.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormDefinition)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormDefinition: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormDefinition();
                if (object.typeId != null)
                    message.typeId = String(object.typeId);
                if (object.alias != null)
                    message.alias = String(object.alias);
                if (object.revision != null)
                    message.revision = object.revision >>> 0;
                if (object.name != null)
                    message.name = String(object.name);
                if (object.description != null)
                    message.description = String(object.description);
                if (object.fields) {
                    if (!Array.isArray(object.fields))
                        throw TypeError(".revault.bindings.FormDefinition.fields: array expected");
                    message.fields = [];
                    for (let i = 0; i < object.fields.length; ++i) {
                        if (!$util.isObject(object.fields[i]))
                            throw TypeError(".revault.bindings.FormDefinition.fields: object expected");
                        message.fields[i] = $root.revault.bindings.FormField.fromObject(object.fields[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FormDefinition message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {revault.bindings.FormDefinition} message FormDefinition
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormDefinition.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.fields = [];
                if (options.defaults) {
                    object.typeId = "";
                    object.alias = "";
                    object.revision = 0;
                    object.name = "";
                    object.description = "";
                }
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    object.typeId = message.typeId;
                if (message.alias != null && Object.hasOwnProperty.call(message, "alias"))
                    object.alias = message.alias;
                if (message.revision != null && Object.hasOwnProperty.call(message, "revision"))
                    object.revision = message.revision;
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                    object.description = message.description;
                if (message.fields && message.fields.length) {
                    object.fields = [];
                    for (let j = 0; j < message.fields.length; ++j)
                        object.fields[j] = $root.revault.bindings.FormField.toObject(message.fields[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FormDefinition to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormDefinition
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormDefinition.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormDefinition
             * @function getTypeUrl
             * @memberof revault.bindings.FormDefinition
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormDefinition.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormDefinition";
            };

            return FormDefinition;
        })();

        bindings.FormDefinitionList = (function() {

            /**
             * Properties of a FormDefinitionList.
             * @memberof revault.bindings
             * @interface IFormDefinitionList
             * @property {Array.<revault.bindings.IFormDefinition>|null} [values] FormDefinitionList values
             */

            /**
             * Constructs a new FormDefinitionList.
             * @memberof revault.bindings
             * @classdesc Represents a FormDefinitionList.
             * @implements IFormDefinitionList
             * @constructor
             * @param {revault.bindings.IFormDefinitionList=} [properties] Properties to set
             */
            function FormDefinitionList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormDefinitionList values.
             * @member {Array.<revault.bindings.IFormDefinition>} values
             * @memberof revault.bindings.FormDefinitionList
             * @instance
             */
            FormDefinitionList.prototype.values = $util.emptyArray;

            /**
             * Creates a new FormDefinitionList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {revault.bindings.IFormDefinitionList=} [properties] Properties to set
             * @returns {revault.bindings.FormDefinitionList} FormDefinitionList instance
             */
            FormDefinitionList.create = function create(properties) {
                return new FormDefinitionList(properties);
            };

            /**
             * Encodes the specified FormDefinitionList message. Does not implicitly {@link revault.bindings.FormDefinitionList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {revault.bindings.IFormDefinitionList} message FormDefinitionList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormDefinitionList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.FormDefinition.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FormDefinitionList message, length delimited. Does not implicitly {@link revault.bindings.FormDefinitionList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {revault.bindings.IFormDefinitionList} message FormDefinitionList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormDefinitionList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormDefinitionList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormDefinitionList} FormDefinitionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormDefinitionList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormDefinitionList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.FormDefinition.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormDefinitionList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormDefinitionList} FormDefinitionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormDefinitionList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormDefinitionList message.
             * @function verify
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormDefinitionList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.FormDefinition.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FormDefinitionList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormDefinitionList} FormDefinitionList
             */
            FormDefinitionList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormDefinitionList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormDefinitionList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormDefinitionList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.FormDefinitionList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.FormDefinitionList.values: object expected");
                        message.values[i] = $root.revault.bindings.FormDefinition.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FormDefinitionList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {revault.bindings.FormDefinitionList} message FormDefinitionList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormDefinitionList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.FormDefinition.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FormDefinitionList to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormDefinitionList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormDefinitionList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormDefinitionList
             * @function getTypeUrl
             * @memberof revault.bindings.FormDefinitionList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormDefinitionList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormDefinitionList";
            };

            return FormDefinitionList;
        })();

        bindings.FormValue = (function() {

            /**
             * Properties of a FormValue.
             * @memberof revault.bindings
             * @interface IFormValue
             * @property {string|null} [fieldId] FormValue fieldId
             * @property {string|null} [label] FormValue label
             * @property {string|null} [kind] FormValue kind
             * @property {string|null} [value] FormValue value
             * @property {boolean|null} [secret] FormValue secret
             */

            /**
             * Constructs a new FormValue.
             * @memberof revault.bindings
             * @classdesc Represents a FormValue.
             * @implements IFormValue
             * @constructor
             * @param {revault.bindings.IFormValue=} [properties] Properties to set
             */
            function FormValue(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormValue fieldId.
             * @member {string} fieldId
             * @memberof revault.bindings.FormValue
             * @instance
             */
            FormValue.prototype.fieldId = "";

            /**
             * FormValue label.
             * @member {string} label
             * @memberof revault.bindings.FormValue
             * @instance
             */
            FormValue.prototype.label = "";

            /**
             * FormValue kind.
             * @member {string} kind
             * @memberof revault.bindings.FormValue
             * @instance
             */
            FormValue.prototype.kind = "";

            /**
             * FormValue value.
             * @member {string} value
             * @memberof revault.bindings.FormValue
             * @instance
             */
            FormValue.prototype.value = "";

            /**
             * FormValue secret.
             * @member {boolean} secret
             * @memberof revault.bindings.FormValue
             * @instance
             */
            FormValue.prototype.secret = false;

            /**
             * Creates a new FormValue instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormValue
             * @static
             * @param {revault.bindings.IFormValue=} [properties] Properties to set
             * @returns {revault.bindings.FormValue} FormValue instance
             */
            FormValue.create = function create(properties) {
                return new FormValue(properties);
            };

            /**
             * Encodes the specified FormValue message. Does not implicitly {@link revault.bindings.FormValue.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormValue
             * @static
             * @param {revault.bindings.IFormValue} message FormValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormValue.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.fieldId != null && Object.hasOwnProperty.call(message, "fieldId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.fieldId);
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.label);
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.kind);
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.value);
                if (message.secret != null && Object.hasOwnProperty.call(message, "secret"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.secret);
                return writer;
            };

            /**
             * Encodes the specified FormValue message, length delimited. Does not implicitly {@link revault.bindings.FormValue.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormValue
             * @static
             * @param {revault.bindings.IFormValue} message FormValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormValue.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormValue message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormValue} FormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormValue.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormValue();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.fieldId = reader.string();
                            break;
                        }
                    case 2: {
                            message.label = reader.string();
                            break;
                        }
                    case 3: {
                            message.kind = reader.string();
                            break;
                        }
                    case 4: {
                            message.value = reader.string();
                            break;
                        }
                    case 5: {
                            message.secret = reader.bool();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormValue message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormValue} FormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormValue.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormValue message.
             * @function verify
             * @memberof revault.bindings.FormValue
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormValue.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.fieldId != null && Object.hasOwnProperty.call(message, "fieldId"))
                    if (!$util.isString(message.fieldId))
                        return "fieldId: string expected";
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    if (!$util.isString(message.kind))
                        return "kind: string expected";
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    if (!$util.isString(message.value))
                        return "value: string expected";
                if (message.secret != null && Object.hasOwnProperty.call(message, "secret"))
                    if (typeof message.secret !== "boolean")
                        return "secret: boolean expected";
                return null;
            };

            /**
             * Creates a FormValue message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormValue
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormValue} FormValue
             */
            FormValue.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormValue)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormValue: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormValue();
                if (object.fieldId != null)
                    message.fieldId = String(object.fieldId);
                if (object.label != null)
                    message.label = String(object.label);
                if (object.kind != null)
                    message.kind = String(object.kind);
                if (object.value != null)
                    message.value = String(object.value);
                if (object.secret != null)
                    message.secret = Boolean(object.secret);
                return message;
            };

            /**
             * Creates a plain object from a FormValue message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormValue
             * @static
             * @param {revault.bindings.FormValue} message FormValue
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormValue.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.fieldId = "";
                    object.label = "";
                    object.kind = "";
                    object.value = "";
                    object.secret = false;
                }
                if (message.fieldId != null && Object.hasOwnProperty.call(message, "fieldId"))
                    object.fieldId = message.fieldId;
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    object.label = message.label;
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    object.kind = message.kind;
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = message.value;
                if (message.secret != null && Object.hasOwnProperty.call(message, "secret"))
                    object.secret = message.secret;
                return object;
            };

            /**
             * Converts this FormValue to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormValue
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormValue.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormValue
             * @function getTypeUrl
             * @memberof revault.bindings.FormValue
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormValue.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormValue";
            };

            return FormValue;
        })();

        bindings.FormRecord = (function() {

            /**
             * Properties of a FormRecord.
             * @memberof revault.bindings
             * @interface IFormRecord
             * @property {string|null} [path] FormRecord path
             * @property {string|null} [name] FormRecord name
             * @property {string|null} [typeId] FormRecord typeId
             * @property {string|null} [definitionAlias] FormRecord definitionAlias
             * @property {number|null} [definitionRevision] FormRecord definitionRevision
             * @property {Array.<revault.bindings.IFormValue>|null} [values] FormRecord values
             */

            /**
             * Constructs a new FormRecord.
             * @memberof revault.bindings
             * @classdesc Represents a FormRecord.
             * @implements IFormRecord
             * @constructor
             * @param {revault.bindings.IFormRecord=} [properties] Properties to set
             */
            function FormRecord(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormRecord path.
             * @member {string} path
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.path = "";

            /**
             * FormRecord name.
             * @member {string} name
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.name = "";

            /**
             * FormRecord typeId.
             * @member {string} typeId
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.typeId = "";

            /**
             * FormRecord definitionAlias.
             * @member {string} definitionAlias
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.definitionAlias = "";

            /**
             * FormRecord definitionRevision.
             * @member {number} definitionRevision
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.definitionRevision = 0;

            /**
             * FormRecord values.
             * @member {Array.<revault.bindings.IFormValue>} values
             * @memberof revault.bindings.FormRecord
             * @instance
             */
            FormRecord.prototype.values = $util.emptyArray;

            /**
             * Creates a new FormRecord instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {revault.bindings.IFormRecord=} [properties] Properties to set
             * @returns {revault.bindings.FormRecord} FormRecord instance
             */
            FormRecord.create = function create(properties) {
                return new FormRecord(properties);
            };

            /**
             * Encodes the specified FormRecord message. Does not implicitly {@link revault.bindings.FormRecord.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {revault.bindings.IFormRecord} message FormRecord message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormRecord.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.path);
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.name);
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.typeId);
                if (message.definitionAlias != null && Object.hasOwnProperty.call(message, "definitionAlias"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.definitionAlias);
                if (message.definitionRevision != null && Object.hasOwnProperty.call(message, "definitionRevision"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint32(message.definitionRevision);
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.FormValue.encode(message.values[i], writer.uint32(/* id 6, wireType 2 =*/50).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FormRecord message, length delimited. Does not implicitly {@link revault.bindings.FormRecord.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {revault.bindings.IFormRecord} message FormRecord message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormRecord.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormRecord message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormRecord} FormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormRecord.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormRecord();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.path = reader.string();
                            break;
                        }
                    case 2: {
                            message.name = reader.string();
                            break;
                        }
                    case 3: {
                            message.typeId = reader.string();
                            break;
                        }
                    case 4: {
                            message.definitionAlias = reader.string();
                            break;
                        }
                    case 5: {
                            message.definitionRevision = reader.uint32();
                            break;
                        }
                    case 6: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.FormValue.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormRecord message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormRecord} FormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormRecord.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormRecord message.
             * @function verify
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormRecord.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    if (!$util.isString(message.path))
                        return "path: string expected";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    if (!$util.isString(message.typeId))
                        return "typeId: string expected";
                if (message.definitionAlias != null && Object.hasOwnProperty.call(message, "definitionAlias"))
                    if (!$util.isString(message.definitionAlias))
                        return "definitionAlias: string expected";
                if (message.definitionRevision != null && Object.hasOwnProperty.call(message, "definitionRevision"))
                    if (!$util.isInteger(message.definitionRevision))
                        return "definitionRevision: integer expected";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.FormValue.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FormRecord message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormRecord} FormRecord
             */
            FormRecord.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormRecord)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormRecord: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormRecord();
                if (object.path != null)
                    message.path = String(object.path);
                if (object.name != null)
                    message.name = String(object.name);
                if (object.typeId != null)
                    message.typeId = String(object.typeId);
                if (object.definitionAlias != null)
                    message.definitionAlias = String(object.definitionAlias);
                if (object.definitionRevision != null)
                    message.definitionRevision = object.definitionRevision >>> 0;
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.FormRecord.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.FormRecord.values: object expected");
                        message.values[i] = $root.revault.bindings.FormValue.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FormRecord message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {revault.bindings.FormRecord} message FormRecord
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormRecord.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (options.defaults) {
                    object.path = "";
                    object.name = "";
                    object.typeId = "";
                    object.definitionAlias = "";
                    object.definitionRevision = 0;
                }
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    object.path = message.path;
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.typeId != null && Object.hasOwnProperty.call(message, "typeId"))
                    object.typeId = message.typeId;
                if (message.definitionAlias != null && Object.hasOwnProperty.call(message, "definitionAlias"))
                    object.definitionAlias = message.definitionAlias;
                if (message.definitionRevision != null && Object.hasOwnProperty.call(message, "definitionRevision"))
                    object.definitionRevision = message.definitionRevision;
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.FormValue.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FormRecord to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormRecord
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormRecord.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormRecord
             * @function getTypeUrl
             * @memberof revault.bindings.FormRecord
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormRecord.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormRecord";
            };

            return FormRecord;
        })();

        bindings.FormRecordList = (function() {

            /**
             * Properties of a FormRecordList.
             * @memberof revault.bindings
             * @interface IFormRecordList
             * @property {Array.<revault.bindings.IFormRecord>|null} [values] FormRecordList values
             */

            /**
             * Constructs a new FormRecordList.
             * @memberof revault.bindings
             * @classdesc Represents a FormRecordList.
             * @implements IFormRecordList
             * @constructor
             * @param {revault.bindings.IFormRecordList=} [properties] Properties to set
             */
            function FormRecordList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FormRecordList values.
             * @member {Array.<revault.bindings.IFormRecord>} values
             * @memberof revault.bindings.FormRecordList
             * @instance
             */
            FormRecordList.prototype.values = $util.emptyArray;

            /**
             * Creates a new FormRecordList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {revault.bindings.IFormRecordList=} [properties] Properties to set
             * @returns {revault.bindings.FormRecordList} FormRecordList instance
             */
            FormRecordList.create = function create(properties) {
                return new FormRecordList(properties);
            };

            /**
             * Encodes the specified FormRecordList message. Does not implicitly {@link revault.bindings.FormRecordList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {revault.bindings.IFormRecordList} message FormRecordList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormRecordList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.FormRecord.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FormRecordList message, length delimited. Does not implicitly {@link revault.bindings.FormRecordList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {revault.bindings.IFormRecordList} message FormRecordList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FormRecordList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FormRecordList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FormRecordList} FormRecordList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormRecordList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FormRecordList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.FormRecord.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FormRecordList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FormRecordList} FormRecordList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FormRecordList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FormRecordList message.
             * @function verify
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FormRecordList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.FormRecord.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FormRecordList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FormRecordList} FormRecordList
             */
            FormRecordList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FormRecordList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FormRecordList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FormRecordList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.FormRecordList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.FormRecordList.values: object expected");
                        message.values[i] = $root.revault.bindings.FormRecord.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FormRecordList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {revault.bindings.FormRecordList} message FormRecordList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FormRecordList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.FormRecord.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FormRecordList to JSON.
             * @function toJSON
             * @memberof revault.bindings.FormRecordList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FormRecordList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FormRecordList
             * @function getTypeUrl
             * @memberof revault.bindings.FormRecordList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FormRecordList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FormRecordList";
            };

            return FormRecordList;
        })();

        bindings.OptionalFormRecord = (function() {

            /**
             * Properties of an OptionalFormRecord.
             * @memberof revault.bindings
             * @interface IOptionalFormRecord
             * @property {revault.bindings.IFormRecord|null} [value] OptionalFormRecord value
             */

            /**
             * Constructs a new OptionalFormRecord.
             * @memberof revault.bindings
             * @classdesc Represents an OptionalFormRecord.
             * @implements IOptionalFormRecord
             * @constructor
             * @param {revault.bindings.IOptionalFormRecord=} [properties] Properties to set
             */
            function OptionalFormRecord(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OptionalFormRecord value.
             * @member {revault.bindings.IFormRecord|null|undefined} value
             * @memberof revault.bindings.OptionalFormRecord
             * @instance
             */
            OptionalFormRecord.prototype.value = null;

            /**
             * Creates a new OptionalFormRecord instance using the specified properties.
             * @function create
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {revault.bindings.IOptionalFormRecord=} [properties] Properties to set
             * @returns {revault.bindings.OptionalFormRecord} OptionalFormRecord instance
             */
            OptionalFormRecord.create = function create(properties) {
                return new OptionalFormRecord(properties);
            };

            /**
             * Encodes the specified OptionalFormRecord message. Does not implicitly {@link revault.bindings.OptionalFormRecord.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {revault.bindings.IOptionalFormRecord} message OptionalFormRecord message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalFormRecord.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    $root.revault.bindings.FormRecord.encode(message.value, writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified OptionalFormRecord message, length delimited. Does not implicitly {@link revault.bindings.OptionalFormRecord.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {revault.bindings.IOptionalFormRecord} message OptionalFormRecord message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalFormRecord.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OptionalFormRecord message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.OptionalFormRecord} OptionalFormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalFormRecord.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.OptionalFormRecord();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.value = $root.revault.bindings.FormRecord.decode(reader, reader.uint32(), undefined, long + 1);
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OptionalFormRecord message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.OptionalFormRecord} OptionalFormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalFormRecord.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OptionalFormRecord message.
             * @function verify
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OptionalFormRecord.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.value != null && Object.hasOwnProperty.call(message, "value")) {
                    let error = $root.revault.bindings.FormRecord.verify(message.value, long + 1);
                    if (error)
                        return "value." + error;
                }
                return null;
            };

            /**
             * Creates an OptionalFormRecord message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.OptionalFormRecord} OptionalFormRecord
             */
            OptionalFormRecord.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.OptionalFormRecord)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.OptionalFormRecord: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.OptionalFormRecord();
                if (object.value != null) {
                    if (!$util.isObject(object.value))
                        throw TypeError(".revault.bindings.OptionalFormRecord.value: object expected");
                    message.value = $root.revault.bindings.FormRecord.fromObject(object.value, long + 1);
                }
                return message;
            };

            /**
             * Creates a plain object from an OptionalFormRecord message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {revault.bindings.OptionalFormRecord} message OptionalFormRecord
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OptionalFormRecord.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults)
                    object.value = null;
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = $root.revault.bindings.FormRecord.toObject(message.value, options, q + 1);
                return object;
            };

            /**
             * Converts this OptionalFormRecord to JSON.
             * @function toJSON
             * @memberof revault.bindings.OptionalFormRecord
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OptionalFormRecord.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for OptionalFormRecord
             * @function getTypeUrl
             * @memberof revault.bindings.OptionalFormRecord
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            OptionalFormRecord.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.OptionalFormRecord";
            };

            return OptionalFormRecord;
        })();

        bindings.OptionalFormValue = (function() {

            /**
             * Properties of an OptionalFormValue.
             * @memberof revault.bindings
             * @interface IOptionalFormValue
             * @property {revault.bindings.IFormValue|null} [value] OptionalFormValue value
             */

            /**
             * Constructs a new OptionalFormValue.
             * @memberof revault.bindings
             * @classdesc Represents an OptionalFormValue.
             * @implements IOptionalFormValue
             * @constructor
             * @param {revault.bindings.IOptionalFormValue=} [properties] Properties to set
             */
            function OptionalFormValue(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OptionalFormValue value.
             * @member {revault.bindings.IFormValue|null|undefined} value
             * @memberof revault.bindings.OptionalFormValue
             * @instance
             */
            OptionalFormValue.prototype.value = null;

            /**
             * Creates a new OptionalFormValue instance using the specified properties.
             * @function create
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {revault.bindings.IOptionalFormValue=} [properties] Properties to set
             * @returns {revault.bindings.OptionalFormValue} OptionalFormValue instance
             */
            OptionalFormValue.create = function create(properties) {
                return new OptionalFormValue(properties);
            };

            /**
             * Encodes the specified OptionalFormValue message. Does not implicitly {@link revault.bindings.OptionalFormValue.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {revault.bindings.IOptionalFormValue} message OptionalFormValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalFormValue.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    $root.revault.bindings.FormValue.encode(message.value, writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified OptionalFormValue message, length delimited. Does not implicitly {@link revault.bindings.OptionalFormValue.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {revault.bindings.IOptionalFormValue} message OptionalFormValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalFormValue.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OptionalFormValue message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.OptionalFormValue} OptionalFormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalFormValue.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.OptionalFormValue();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.value = $root.revault.bindings.FormValue.decode(reader, reader.uint32(), undefined, long + 1);
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OptionalFormValue message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.OptionalFormValue} OptionalFormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalFormValue.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OptionalFormValue message.
             * @function verify
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OptionalFormValue.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.value != null && Object.hasOwnProperty.call(message, "value")) {
                    let error = $root.revault.bindings.FormValue.verify(message.value, long + 1);
                    if (error)
                        return "value." + error;
                }
                return null;
            };

            /**
             * Creates an OptionalFormValue message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.OptionalFormValue} OptionalFormValue
             */
            OptionalFormValue.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.OptionalFormValue)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.OptionalFormValue: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.OptionalFormValue();
                if (object.value != null) {
                    if (!$util.isObject(object.value))
                        throw TypeError(".revault.bindings.OptionalFormValue.value: object expected");
                    message.value = $root.revault.bindings.FormValue.fromObject(object.value, long + 1);
                }
                return message;
            };

            /**
             * Creates a plain object from an OptionalFormValue message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {revault.bindings.OptionalFormValue} message OptionalFormValue
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OptionalFormValue.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults)
                    object.value = null;
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = $root.revault.bindings.FormValue.toObject(message.value, options, q + 1);
                return object;
            };

            /**
             * Converts this OptionalFormValue to JSON.
             * @function toJSON
             * @memberof revault.bindings.OptionalFormValue
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OptionalFormValue.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for OptionalFormValue
             * @function getTypeUrl
             * @memberof revault.bindings.OptionalFormValue
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            OptionalFormValue.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.OptionalFormValue";
            };

            return OptionalFormValue;
        })();

        bindings.RecoveryReport = (function() {

            /**
             * Properties of a RecoveryReport.
             * @memberof revault.bindings
             * @interface IRecoveryReport
             * @property {Array.<revault.bindings.ILockboxEntry>|null} [intactFiles] RecoveryReport intactFiles
             * @property {number|Long|null} [intactFileCount] RecoveryReport intactFileCount
             * @property {number|Long|null} [partialFiles] RecoveryReport partialFiles
             * @property {number|Long|null} [corruptRecords] RecoveryReport corruptRecords
             * @property {boolean|null} [tocRecovered] RecoveryReport tocRecovered
             * @property {boolean|null} [variablesRecovered] RecoveryReport variablesRecovered
             * @property {number|Long|null} [variableCount] RecoveryReport variableCount
             * @property {boolean|null} [formsRecovered] RecoveryReport formsRecovered
             * @property {number|Long|null} [formDefinitionCount] RecoveryReport formDefinitionCount
             * @property {number|Long|null} [formRecordCount] RecoveryReport formRecordCount
             */

            /**
             * Constructs a new RecoveryReport.
             * @memberof revault.bindings
             * @classdesc Represents a RecoveryReport.
             * @implements IRecoveryReport
             * @constructor
             * @param {revault.bindings.IRecoveryReport=} [properties] Properties to set
             */
            function RecoveryReport(properties) {
                this.intactFiles = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * RecoveryReport intactFiles.
             * @member {Array.<revault.bindings.ILockboxEntry>} intactFiles
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.intactFiles = $util.emptyArray;

            /**
             * RecoveryReport intactFileCount.
             * @member {number|Long} intactFileCount
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.intactFileCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * RecoveryReport partialFiles.
             * @member {number|Long} partialFiles
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.partialFiles = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * RecoveryReport corruptRecords.
             * @member {number|Long} corruptRecords
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.corruptRecords = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * RecoveryReport tocRecovered.
             * @member {boolean} tocRecovered
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.tocRecovered = false;

            /**
             * RecoveryReport variablesRecovered.
             * @member {boolean} variablesRecovered
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.variablesRecovered = false;

            /**
             * RecoveryReport variableCount.
             * @member {number|Long} variableCount
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.variableCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * RecoveryReport formsRecovered.
             * @member {boolean} formsRecovered
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.formsRecovered = false;

            /**
             * RecoveryReport formDefinitionCount.
             * @member {number|Long} formDefinitionCount
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.formDefinitionCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * RecoveryReport formRecordCount.
             * @member {number|Long} formRecordCount
             * @memberof revault.bindings.RecoveryReport
             * @instance
             */
            RecoveryReport.prototype.formRecordCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new RecoveryReport instance using the specified properties.
             * @function create
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {revault.bindings.IRecoveryReport=} [properties] Properties to set
             * @returns {revault.bindings.RecoveryReport} RecoveryReport instance
             */
            RecoveryReport.create = function create(properties) {
                return new RecoveryReport(properties);
            };

            /**
             * Encodes the specified RecoveryReport message. Does not implicitly {@link revault.bindings.RecoveryReport.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {revault.bindings.IRecoveryReport} message RecoveryReport message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryReport.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.intactFiles != null && message.intactFiles.length)
                    for (let i = 0; i < message.intactFiles.length; ++i)
                        $root.revault.bindings.LockboxEntry.encode(message.intactFiles[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                if (message.intactFileCount != null && Object.hasOwnProperty.call(message, "intactFileCount"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.intactFileCount);
                if (message.partialFiles != null && Object.hasOwnProperty.call(message, "partialFiles"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.partialFiles);
                if (message.corruptRecords != null && Object.hasOwnProperty.call(message, "corruptRecords"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.corruptRecords);
                if (message.tocRecovered != null && Object.hasOwnProperty.call(message, "tocRecovered"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.tocRecovered);
                if (message.variablesRecovered != null && Object.hasOwnProperty.call(message, "variablesRecovered"))
                    writer.uint32(/* id 6, wireType 0 =*/48).bool(message.variablesRecovered);
                if (message.variableCount != null && Object.hasOwnProperty.call(message, "variableCount"))
                    writer.uint32(/* id 7, wireType 0 =*/56).uint64(message.variableCount);
                if (message.formsRecovered != null && Object.hasOwnProperty.call(message, "formsRecovered"))
                    writer.uint32(/* id 8, wireType 0 =*/64).bool(message.formsRecovered);
                if (message.formDefinitionCount != null && Object.hasOwnProperty.call(message, "formDefinitionCount"))
                    writer.uint32(/* id 9, wireType 0 =*/72).uint64(message.formDefinitionCount);
                if (message.formRecordCount != null && Object.hasOwnProperty.call(message, "formRecordCount"))
                    writer.uint32(/* id 10, wireType 0 =*/80).uint64(message.formRecordCount);
                return writer;
            };

            /**
             * Encodes the specified RecoveryReport message, length delimited. Does not implicitly {@link revault.bindings.RecoveryReport.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {revault.bindings.IRecoveryReport} message RecoveryReport message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryReport.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a RecoveryReport message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.RecoveryReport} RecoveryReport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryReport.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.RecoveryReport();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.intactFiles && message.intactFiles.length))
                                message.intactFiles = [];
                            message.intactFiles.push($root.revault.bindings.LockboxEntry.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    case 2: {
                            message.intactFileCount = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.partialFiles = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.corruptRecords = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.tocRecovered = reader.bool();
                            break;
                        }
                    case 6: {
                            message.variablesRecovered = reader.bool();
                            break;
                        }
                    case 7: {
                            message.variableCount = reader.uint64();
                            break;
                        }
                    case 8: {
                            message.formsRecovered = reader.bool();
                            break;
                        }
                    case 9: {
                            message.formDefinitionCount = reader.uint64();
                            break;
                        }
                    case 10: {
                            message.formRecordCount = reader.uint64();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a RecoveryReport message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.RecoveryReport} RecoveryReport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryReport.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a RecoveryReport message.
             * @function verify
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RecoveryReport.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.intactFiles != null && Object.hasOwnProperty.call(message, "intactFiles")) {
                    if (!Array.isArray(message.intactFiles))
                        return "intactFiles: array expected";
                    for (let i = 0; i < message.intactFiles.length; ++i) {
                        let error = $root.revault.bindings.LockboxEntry.verify(message.intactFiles[i], long + 1);
                        if (error)
                            return "intactFiles." + error;
                    }
                }
                if (message.intactFileCount != null && Object.hasOwnProperty.call(message, "intactFileCount"))
                    if (!$util.isInteger(message.intactFileCount) && !(message.intactFileCount && $util.isInteger(message.intactFileCount.low) && $util.isInteger(message.intactFileCount.high)))
                        return "intactFileCount: integer|Long expected";
                if (message.partialFiles != null && Object.hasOwnProperty.call(message, "partialFiles"))
                    if (!$util.isInteger(message.partialFiles) && !(message.partialFiles && $util.isInteger(message.partialFiles.low) && $util.isInteger(message.partialFiles.high)))
                        return "partialFiles: integer|Long expected";
                if (message.corruptRecords != null && Object.hasOwnProperty.call(message, "corruptRecords"))
                    if (!$util.isInteger(message.corruptRecords) && !(message.corruptRecords && $util.isInteger(message.corruptRecords.low) && $util.isInteger(message.corruptRecords.high)))
                        return "corruptRecords: integer|Long expected";
                if (message.tocRecovered != null && Object.hasOwnProperty.call(message, "tocRecovered"))
                    if (typeof message.tocRecovered !== "boolean")
                        return "tocRecovered: boolean expected";
                if (message.variablesRecovered != null && Object.hasOwnProperty.call(message, "variablesRecovered"))
                    if (typeof message.variablesRecovered !== "boolean")
                        return "variablesRecovered: boolean expected";
                if (message.variableCount != null && Object.hasOwnProperty.call(message, "variableCount"))
                    if (!$util.isInteger(message.variableCount) && !(message.variableCount && $util.isInteger(message.variableCount.low) && $util.isInteger(message.variableCount.high)))
                        return "variableCount: integer|Long expected";
                if (message.formsRecovered != null && Object.hasOwnProperty.call(message, "formsRecovered"))
                    if (typeof message.formsRecovered !== "boolean")
                        return "formsRecovered: boolean expected";
                if (message.formDefinitionCount != null && Object.hasOwnProperty.call(message, "formDefinitionCount"))
                    if (!$util.isInteger(message.formDefinitionCount) && !(message.formDefinitionCount && $util.isInteger(message.formDefinitionCount.low) && $util.isInteger(message.formDefinitionCount.high)))
                        return "formDefinitionCount: integer|Long expected";
                if (message.formRecordCount != null && Object.hasOwnProperty.call(message, "formRecordCount"))
                    if (!$util.isInteger(message.formRecordCount) && !(message.formRecordCount && $util.isInteger(message.formRecordCount.low) && $util.isInteger(message.formRecordCount.high)))
                        return "formRecordCount: integer|Long expected";
                return null;
            };

            /**
             * Creates a RecoveryReport message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.RecoveryReport} RecoveryReport
             */
            RecoveryReport.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.RecoveryReport)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.RecoveryReport: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.RecoveryReport();
                if (object.intactFiles) {
                    if (!Array.isArray(object.intactFiles))
                        throw TypeError(".revault.bindings.RecoveryReport.intactFiles: array expected");
                    message.intactFiles = [];
                    for (let i = 0; i < object.intactFiles.length; ++i) {
                        if (!$util.isObject(object.intactFiles[i]))
                            throw TypeError(".revault.bindings.RecoveryReport.intactFiles: object expected");
                        message.intactFiles[i] = $root.revault.bindings.LockboxEntry.fromObject(object.intactFiles[i], long + 1);
                    }
                }
                if (object.intactFileCount != null)
                    if ($util.Long)
                        message.intactFileCount = $util.Long.fromValue(object.intactFileCount, true);
                    else if (typeof object.intactFileCount === "string")
                        message.intactFileCount = parseInt(object.intactFileCount, 10);
                    else if (typeof object.intactFileCount === "number")
                        message.intactFileCount = object.intactFileCount;
                    else if (typeof object.intactFileCount === "object")
                        message.intactFileCount = new $util.LongBits(object.intactFileCount.low >>> 0, object.intactFileCount.high >>> 0).toNumber(true);
                if (object.partialFiles != null)
                    if ($util.Long)
                        message.partialFiles = $util.Long.fromValue(object.partialFiles, true);
                    else if (typeof object.partialFiles === "string")
                        message.partialFiles = parseInt(object.partialFiles, 10);
                    else if (typeof object.partialFiles === "number")
                        message.partialFiles = object.partialFiles;
                    else if (typeof object.partialFiles === "object")
                        message.partialFiles = new $util.LongBits(object.partialFiles.low >>> 0, object.partialFiles.high >>> 0).toNumber(true);
                if (object.corruptRecords != null)
                    if ($util.Long)
                        message.corruptRecords = $util.Long.fromValue(object.corruptRecords, true);
                    else if (typeof object.corruptRecords === "string")
                        message.corruptRecords = parseInt(object.corruptRecords, 10);
                    else if (typeof object.corruptRecords === "number")
                        message.corruptRecords = object.corruptRecords;
                    else if (typeof object.corruptRecords === "object")
                        message.corruptRecords = new $util.LongBits(object.corruptRecords.low >>> 0, object.corruptRecords.high >>> 0).toNumber(true);
                if (object.tocRecovered != null)
                    message.tocRecovered = Boolean(object.tocRecovered);
                if (object.variablesRecovered != null)
                    message.variablesRecovered = Boolean(object.variablesRecovered);
                if (object.variableCount != null)
                    if ($util.Long)
                        message.variableCount = $util.Long.fromValue(object.variableCount, true);
                    else if (typeof object.variableCount === "string")
                        message.variableCount = parseInt(object.variableCount, 10);
                    else if (typeof object.variableCount === "number")
                        message.variableCount = object.variableCount;
                    else if (typeof object.variableCount === "object")
                        message.variableCount = new $util.LongBits(object.variableCount.low >>> 0, object.variableCount.high >>> 0).toNumber(true);
                if (object.formsRecovered != null)
                    message.formsRecovered = Boolean(object.formsRecovered);
                if (object.formDefinitionCount != null)
                    if ($util.Long)
                        message.formDefinitionCount = $util.Long.fromValue(object.formDefinitionCount, true);
                    else if (typeof object.formDefinitionCount === "string")
                        message.formDefinitionCount = parseInt(object.formDefinitionCount, 10);
                    else if (typeof object.formDefinitionCount === "number")
                        message.formDefinitionCount = object.formDefinitionCount;
                    else if (typeof object.formDefinitionCount === "object")
                        message.formDefinitionCount = new $util.LongBits(object.formDefinitionCount.low >>> 0, object.formDefinitionCount.high >>> 0).toNumber(true);
                if (object.formRecordCount != null)
                    if ($util.Long)
                        message.formRecordCount = $util.Long.fromValue(object.formRecordCount, true);
                    else if (typeof object.formRecordCount === "string")
                        message.formRecordCount = parseInt(object.formRecordCount, 10);
                    else if (typeof object.formRecordCount === "number")
                        message.formRecordCount = object.formRecordCount;
                    else if (typeof object.formRecordCount === "object")
                        message.formRecordCount = new $util.LongBits(object.formRecordCount.low >>> 0, object.formRecordCount.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from a RecoveryReport message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {revault.bindings.RecoveryReport} message RecoveryReport
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RecoveryReport.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.intactFiles = [];
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.intactFileCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.intactFileCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.partialFiles = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.partialFiles = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.corruptRecords = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.corruptRecords = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.tocRecovered = false;
                    object.variablesRecovered = false;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.variableCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.variableCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.formsRecovered = false;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.formDefinitionCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.formDefinitionCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.formRecordCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.formRecordCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.intactFiles && message.intactFiles.length) {
                    object.intactFiles = [];
                    for (let j = 0; j < message.intactFiles.length; ++j)
                        object.intactFiles[j] = $root.revault.bindings.LockboxEntry.toObject(message.intactFiles[j], options, q + 1);
                }
                if (message.intactFileCount != null && Object.hasOwnProperty.call(message, "intactFileCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.intactFileCount = typeof message.intactFileCount === "number" ? BigInt(message.intactFileCount) : $util.Long.fromBits(message.intactFileCount.low >>> 0, message.intactFileCount.high >>> 0, true).toBigInt();
                    else if (typeof message.intactFileCount === "number")
                        object.intactFileCount = options.longs === String ? String(message.intactFileCount) : message.intactFileCount;
                    else
                        object.intactFileCount = options.longs === String ? $util.Long.prototype.toString.call(message.intactFileCount) : options.longs === Number ? new $util.LongBits(message.intactFileCount.low >>> 0, message.intactFileCount.high >>> 0).toNumber(true) : message.intactFileCount;
                if (message.partialFiles != null && Object.hasOwnProperty.call(message, "partialFiles"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.partialFiles = typeof message.partialFiles === "number" ? BigInt(message.partialFiles) : $util.Long.fromBits(message.partialFiles.low >>> 0, message.partialFiles.high >>> 0, true).toBigInt();
                    else if (typeof message.partialFiles === "number")
                        object.partialFiles = options.longs === String ? String(message.partialFiles) : message.partialFiles;
                    else
                        object.partialFiles = options.longs === String ? $util.Long.prototype.toString.call(message.partialFiles) : options.longs === Number ? new $util.LongBits(message.partialFiles.low >>> 0, message.partialFiles.high >>> 0).toNumber(true) : message.partialFiles;
                if (message.corruptRecords != null && Object.hasOwnProperty.call(message, "corruptRecords"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.corruptRecords = typeof message.corruptRecords === "number" ? BigInt(message.corruptRecords) : $util.Long.fromBits(message.corruptRecords.low >>> 0, message.corruptRecords.high >>> 0, true).toBigInt();
                    else if (typeof message.corruptRecords === "number")
                        object.corruptRecords = options.longs === String ? String(message.corruptRecords) : message.corruptRecords;
                    else
                        object.corruptRecords = options.longs === String ? $util.Long.prototype.toString.call(message.corruptRecords) : options.longs === Number ? new $util.LongBits(message.corruptRecords.low >>> 0, message.corruptRecords.high >>> 0).toNumber(true) : message.corruptRecords;
                if (message.tocRecovered != null && Object.hasOwnProperty.call(message, "tocRecovered"))
                    object.tocRecovered = message.tocRecovered;
                if (message.variablesRecovered != null && Object.hasOwnProperty.call(message, "variablesRecovered"))
                    object.variablesRecovered = message.variablesRecovered;
                if (message.variableCount != null && Object.hasOwnProperty.call(message, "variableCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.variableCount = typeof message.variableCount === "number" ? BigInt(message.variableCount) : $util.Long.fromBits(message.variableCount.low >>> 0, message.variableCount.high >>> 0, true).toBigInt();
                    else if (typeof message.variableCount === "number")
                        object.variableCount = options.longs === String ? String(message.variableCount) : message.variableCount;
                    else
                        object.variableCount = options.longs === String ? $util.Long.prototype.toString.call(message.variableCount) : options.longs === Number ? new $util.LongBits(message.variableCount.low >>> 0, message.variableCount.high >>> 0).toNumber(true) : message.variableCount;
                if (message.formsRecovered != null && Object.hasOwnProperty.call(message, "formsRecovered"))
                    object.formsRecovered = message.formsRecovered;
                if (message.formDefinitionCount != null && Object.hasOwnProperty.call(message, "formDefinitionCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.formDefinitionCount = typeof message.formDefinitionCount === "number" ? BigInt(message.formDefinitionCount) : $util.Long.fromBits(message.formDefinitionCount.low >>> 0, message.formDefinitionCount.high >>> 0, true).toBigInt();
                    else if (typeof message.formDefinitionCount === "number")
                        object.formDefinitionCount = options.longs === String ? String(message.formDefinitionCount) : message.formDefinitionCount;
                    else
                        object.formDefinitionCount = options.longs === String ? $util.Long.prototype.toString.call(message.formDefinitionCount) : options.longs === Number ? new $util.LongBits(message.formDefinitionCount.low >>> 0, message.formDefinitionCount.high >>> 0).toNumber(true) : message.formDefinitionCount;
                if (message.formRecordCount != null && Object.hasOwnProperty.call(message, "formRecordCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.formRecordCount = typeof message.formRecordCount === "number" ? BigInt(message.formRecordCount) : $util.Long.fromBits(message.formRecordCount.low >>> 0, message.formRecordCount.high >>> 0, true).toBigInt();
                    else if (typeof message.formRecordCount === "number")
                        object.formRecordCount = options.longs === String ? String(message.formRecordCount) : message.formRecordCount;
                    else
                        object.formRecordCount = options.longs === String ? $util.Long.prototype.toString.call(message.formRecordCount) : options.longs === Number ? new $util.LongBits(message.formRecordCount.low >>> 0, message.formRecordCount.high >>> 0).toNumber(true) : message.formRecordCount;
                return object;
            };

            /**
             * Converts this RecoveryReport to JSON.
             * @function toJSON
             * @memberof revault.bindings.RecoveryReport
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RecoveryReport.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for RecoveryReport
             * @function getTypeUrl
             * @memberof revault.bindings.RecoveryReport
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            RecoveryReport.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.RecoveryReport";
            };

            return RecoveryReport;
        })();

        bindings.KeySlot = (function() {

            /**
             * Properties of a KeySlot.
             * @memberof revault.bindings
             * @interface IKeySlot
             * @property {number|Long|null} [id] KeySlot id
             * @property {string|null} [protection] KeySlot protection
             * @property {string|null} [algorithm] KeySlot algorithm
             */

            /**
             * Constructs a new KeySlot.
             * @memberof revault.bindings
             * @classdesc Represents a KeySlot.
             * @implements IKeySlot
             * @constructor
             * @param {revault.bindings.IKeySlot=} [properties] Properties to set
             */
            function KeySlot(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * KeySlot id.
             * @member {number|Long} id
             * @memberof revault.bindings.KeySlot
             * @instance
             */
            KeySlot.prototype.id = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * KeySlot protection.
             * @member {string} protection
             * @memberof revault.bindings.KeySlot
             * @instance
             */
            KeySlot.prototype.protection = "";

            /**
             * KeySlot algorithm.
             * @member {string} algorithm
             * @memberof revault.bindings.KeySlot
             * @instance
             */
            KeySlot.prototype.algorithm = "";

            /**
             * Creates a new KeySlot instance using the specified properties.
             * @function create
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {revault.bindings.IKeySlot=} [properties] Properties to set
             * @returns {revault.bindings.KeySlot} KeySlot instance
             */
            KeySlot.create = function create(properties) {
                return new KeySlot(properties);
            };

            /**
             * Encodes the specified KeySlot message. Does not implicitly {@link revault.bindings.KeySlot.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {revault.bindings.IKeySlot} message KeySlot message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeySlot.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.id);
                if (message.protection != null && Object.hasOwnProperty.call(message, "protection"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.protection);
                if (message.algorithm != null && Object.hasOwnProperty.call(message, "algorithm"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.algorithm);
                return writer;
            };

            /**
             * Encodes the specified KeySlot message, length delimited. Does not implicitly {@link revault.bindings.KeySlot.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {revault.bindings.IKeySlot} message KeySlot message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeySlot.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a KeySlot message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.KeySlot} KeySlot
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeySlot.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.KeySlot();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.id = reader.uint64();
                            break;
                        }
                    case 2: {
                            message.protection = reader.string();
                            break;
                        }
                    case 3: {
                            message.algorithm = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a KeySlot message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.KeySlot} KeySlot
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeySlot.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a KeySlot message.
             * @function verify
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            KeySlot.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (!$util.isInteger(message.id) && !(message.id && $util.isInteger(message.id.low) && $util.isInteger(message.id.high)))
                        return "id: integer|Long expected";
                if (message.protection != null && Object.hasOwnProperty.call(message, "protection"))
                    if (!$util.isString(message.protection))
                        return "protection: string expected";
                if (message.algorithm != null && Object.hasOwnProperty.call(message, "algorithm"))
                    if (!$util.isString(message.algorithm))
                        return "algorithm: string expected";
                return null;
            };

            /**
             * Creates a KeySlot message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.KeySlot} KeySlot
             */
            KeySlot.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.KeySlot)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.KeySlot: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.KeySlot();
                if (object.id != null)
                    if ($util.Long)
                        message.id = $util.Long.fromValue(object.id, true);
                    else if (typeof object.id === "string")
                        message.id = parseInt(object.id, 10);
                    else if (typeof object.id === "number")
                        message.id = object.id;
                    else if (typeof object.id === "object")
                        message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber(true);
                if (object.protection != null)
                    message.protection = String(object.protection);
                if (object.algorithm != null)
                    message.algorithm = String(object.algorithm);
                return message;
            };

            /**
             * Creates a plain object from a KeySlot message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {revault.bindings.KeySlot} message KeySlot
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            KeySlot.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.id = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.protection = "";
                    object.algorithm = "";
                }
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.id = typeof message.id === "number" ? BigInt(message.id) : $util.Long.fromBits(message.id.low >>> 0, message.id.high >>> 0, true).toBigInt();
                    else if (typeof message.id === "number")
                        object.id = options.longs === String ? String(message.id) : message.id;
                    else
                        object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber(true) : message.id;
                if (message.protection != null && Object.hasOwnProperty.call(message, "protection"))
                    object.protection = message.protection;
                if (message.algorithm != null && Object.hasOwnProperty.call(message, "algorithm"))
                    object.algorithm = message.algorithm;
                return object;
            };

            /**
             * Converts this KeySlot to JSON.
             * @function toJSON
             * @memberof revault.bindings.KeySlot
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            KeySlot.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for KeySlot
             * @function getTypeUrl
             * @memberof revault.bindings.KeySlot
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            KeySlot.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.KeySlot";
            };

            return KeySlot;
        })();

        bindings.KeySlotList = (function() {

            /**
             * Properties of a KeySlotList.
             * @memberof revault.bindings
             * @interface IKeySlotList
             * @property {Array.<revault.bindings.IKeySlot>|null} [values] KeySlotList values
             */

            /**
             * Constructs a new KeySlotList.
             * @memberof revault.bindings
             * @classdesc Represents a KeySlotList.
             * @implements IKeySlotList
             * @constructor
             * @param {revault.bindings.IKeySlotList=} [properties] Properties to set
             */
            function KeySlotList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * KeySlotList values.
             * @member {Array.<revault.bindings.IKeySlot>} values
             * @memberof revault.bindings.KeySlotList
             * @instance
             */
            KeySlotList.prototype.values = $util.emptyArray;

            /**
             * Creates a new KeySlotList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {revault.bindings.IKeySlotList=} [properties] Properties to set
             * @returns {revault.bindings.KeySlotList} KeySlotList instance
             */
            KeySlotList.create = function create(properties) {
                return new KeySlotList(properties);
            };

            /**
             * Encodes the specified KeySlotList message. Does not implicitly {@link revault.bindings.KeySlotList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {revault.bindings.IKeySlotList} message KeySlotList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeySlotList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.KeySlot.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified KeySlotList message, length delimited. Does not implicitly {@link revault.bindings.KeySlotList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {revault.bindings.IKeySlotList} message KeySlotList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeySlotList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a KeySlotList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.KeySlotList} KeySlotList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeySlotList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.KeySlotList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.KeySlot.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a KeySlotList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.KeySlotList} KeySlotList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeySlotList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a KeySlotList message.
             * @function verify
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            KeySlotList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.KeySlot.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a KeySlotList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.KeySlotList} KeySlotList
             */
            KeySlotList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.KeySlotList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.KeySlotList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.KeySlotList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.KeySlotList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.KeySlotList.values: object expected");
                        message.values[i] = $root.revault.bindings.KeySlot.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a KeySlotList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {revault.bindings.KeySlotList} message KeySlotList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            KeySlotList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.KeySlot.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this KeySlotList to JSON.
             * @function toJSON
             * @memberof revault.bindings.KeySlotList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            KeySlotList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for KeySlotList
             * @function getTypeUrl
             * @memberof revault.bindings.KeySlotList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            KeySlotList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.KeySlotList";
            };

            return KeySlotList;
        })();

        bindings.CacheStats = (function() {

            /**
             * Properties of a CacheStats.
             * @memberof revault.bindings
             * @interface ICacheStats
             * @property {number|Long|null} [limitBytes] CacheStats limitBytes
             * @property {number|Long|null} [usedBytes] CacheStats usedBytes
             * @property {number|Long|null} [entries] CacheStats entries
             * @property {number|Long|null} [hits] CacheStats hits
             * @property {number|Long|null} [misses] CacheStats misses
             */

            /**
             * Constructs a new CacheStats.
             * @memberof revault.bindings
             * @classdesc Represents a CacheStats.
             * @implements ICacheStats
             * @constructor
             * @param {revault.bindings.ICacheStats=} [properties] Properties to set
             */
            function CacheStats(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * CacheStats limitBytes.
             * @member {number|Long} limitBytes
             * @memberof revault.bindings.CacheStats
             * @instance
             */
            CacheStats.prototype.limitBytes = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * CacheStats usedBytes.
             * @member {number|Long} usedBytes
             * @memberof revault.bindings.CacheStats
             * @instance
             */
            CacheStats.prototype.usedBytes = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * CacheStats entries.
             * @member {number|Long} entries
             * @memberof revault.bindings.CacheStats
             * @instance
             */
            CacheStats.prototype.entries = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * CacheStats hits.
             * @member {number|Long} hits
             * @memberof revault.bindings.CacheStats
             * @instance
             */
            CacheStats.prototype.hits = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * CacheStats misses.
             * @member {number|Long} misses
             * @memberof revault.bindings.CacheStats
             * @instance
             */
            CacheStats.prototype.misses = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new CacheStats instance using the specified properties.
             * @function create
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {revault.bindings.ICacheStats=} [properties] Properties to set
             * @returns {revault.bindings.CacheStats} CacheStats instance
             */
            CacheStats.create = function create(properties) {
                return new CacheStats(properties);
            };

            /**
             * Encodes the specified CacheStats message. Does not implicitly {@link revault.bindings.CacheStats.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {revault.bindings.ICacheStats} message CacheStats message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CacheStats.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.limitBytes != null && Object.hasOwnProperty.call(message, "limitBytes"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.limitBytes);
                if (message.usedBytes != null && Object.hasOwnProperty.call(message, "usedBytes"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.usedBytes);
                if (message.entries != null && Object.hasOwnProperty.call(message, "entries"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.entries);
                if (message.hits != null && Object.hasOwnProperty.call(message, "hits"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.hits);
                if (message.misses != null && Object.hasOwnProperty.call(message, "misses"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.misses);
                return writer;
            };

            /**
             * Encodes the specified CacheStats message, length delimited. Does not implicitly {@link revault.bindings.CacheStats.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {revault.bindings.ICacheStats} message CacheStats message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CacheStats.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a CacheStats message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.CacheStats} CacheStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CacheStats.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.CacheStats();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.limitBytes = reader.uint64();
                            break;
                        }
                    case 2: {
                            message.usedBytes = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.entries = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.hits = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.misses = reader.uint64();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a CacheStats message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.CacheStats} CacheStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CacheStats.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a CacheStats message.
             * @function verify
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            CacheStats.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.limitBytes != null && Object.hasOwnProperty.call(message, "limitBytes"))
                    if (!$util.isInteger(message.limitBytes) && !(message.limitBytes && $util.isInteger(message.limitBytes.low) && $util.isInteger(message.limitBytes.high)))
                        return "limitBytes: integer|Long expected";
                if (message.usedBytes != null && Object.hasOwnProperty.call(message, "usedBytes"))
                    if (!$util.isInteger(message.usedBytes) && !(message.usedBytes && $util.isInteger(message.usedBytes.low) && $util.isInteger(message.usedBytes.high)))
                        return "usedBytes: integer|Long expected";
                if (message.entries != null && Object.hasOwnProperty.call(message, "entries"))
                    if (!$util.isInteger(message.entries) && !(message.entries && $util.isInteger(message.entries.low) && $util.isInteger(message.entries.high)))
                        return "entries: integer|Long expected";
                if (message.hits != null && Object.hasOwnProperty.call(message, "hits"))
                    if (!$util.isInteger(message.hits) && !(message.hits && $util.isInteger(message.hits.low) && $util.isInteger(message.hits.high)))
                        return "hits: integer|Long expected";
                if (message.misses != null && Object.hasOwnProperty.call(message, "misses"))
                    if (!$util.isInteger(message.misses) && !(message.misses && $util.isInteger(message.misses.low) && $util.isInteger(message.misses.high)))
                        return "misses: integer|Long expected";
                return null;
            };

            /**
             * Creates a CacheStats message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.CacheStats} CacheStats
             */
            CacheStats.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.CacheStats)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.CacheStats: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.CacheStats();
                if (object.limitBytes != null)
                    if ($util.Long)
                        message.limitBytes = $util.Long.fromValue(object.limitBytes, true);
                    else if (typeof object.limitBytes === "string")
                        message.limitBytes = parseInt(object.limitBytes, 10);
                    else if (typeof object.limitBytes === "number")
                        message.limitBytes = object.limitBytes;
                    else if (typeof object.limitBytes === "object")
                        message.limitBytes = new $util.LongBits(object.limitBytes.low >>> 0, object.limitBytes.high >>> 0).toNumber(true);
                if (object.usedBytes != null)
                    if ($util.Long)
                        message.usedBytes = $util.Long.fromValue(object.usedBytes, true);
                    else if (typeof object.usedBytes === "string")
                        message.usedBytes = parseInt(object.usedBytes, 10);
                    else if (typeof object.usedBytes === "number")
                        message.usedBytes = object.usedBytes;
                    else if (typeof object.usedBytes === "object")
                        message.usedBytes = new $util.LongBits(object.usedBytes.low >>> 0, object.usedBytes.high >>> 0).toNumber(true);
                if (object.entries != null)
                    if ($util.Long)
                        message.entries = $util.Long.fromValue(object.entries, true);
                    else if (typeof object.entries === "string")
                        message.entries = parseInt(object.entries, 10);
                    else if (typeof object.entries === "number")
                        message.entries = object.entries;
                    else if (typeof object.entries === "object")
                        message.entries = new $util.LongBits(object.entries.low >>> 0, object.entries.high >>> 0).toNumber(true);
                if (object.hits != null)
                    if ($util.Long)
                        message.hits = $util.Long.fromValue(object.hits, true);
                    else if (typeof object.hits === "string")
                        message.hits = parseInt(object.hits, 10);
                    else if (typeof object.hits === "number")
                        message.hits = object.hits;
                    else if (typeof object.hits === "object")
                        message.hits = new $util.LongBits(object.hits.low >>> 0, object.hits.high >>> 0).toNumber(true);
                if (object.misses != null)
                    if ($util.Long)
                        message.misses = $util.Long.fromValue(object.misses, true);
                    else if (typeof object.misses === "string")
                        message.misses = parseInt(object.misses, 10);
                    else if (typeof object.misses === "number")
                        message.misses = object.misses;
                    else if (typeof object.misses === "object")
                        message.misses = new $util.LongBits(object.misses.low >>> 0, object.misses.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from a CacheStats message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {revault.bindings.CacheStats} message CacheStats
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            CacheStats.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.limitBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.limitBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.usedBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.usedBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.entries = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.entries = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.hits = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.hits = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.misses = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.misses = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.limitBytes != null && Object.hasOwnProperty.call(message, "limitBytes"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.limitBytes = typeof message.limitBytes === "number" ? BigInt(message.limitBytes) : $util.Long.fromBits(message.limitBytes.low >>> 0, message.limitBytes.high >>> 0, true).toBigInt();
                    else if (typeof message.limitBytes === "number")
                        object.limitBytes = options.longs === String ? String(message.limitBytes) : message.limitBytes;
                    else
                        object.limitBytes = options.longs === String ? $util.Long.prototype.toString.call(message.limitBytes) : options.longs === Number ? new $util.LongBits(message.limitBytes.low >>> 0, message.limitBytes.high >>> 0).toNumber(true) : message.limitBytes;
                if (message.usedBytes != null && Object.hasOwnProperty.call(message, "usedBytes"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.usedBytes = typeof message.usedBytes === "number" ? BigInt(message.usedBytes) : $util.Long.fromBits(message.usedBytes.low >>> 0, message.usedBytes.high >>> 0, true).toBigInt();
                    else if (typeof message.usedBytes === "number")
                        object.usedBytes = options.longs === String ? String(message.usedBytes) : message.usedBytes;
                    else
                        object.usedBytes = options.longs === String ? $util.Long.prototype.toString.call(message.usedBytes) : options.longs === Number ? new $util.LongBits(message.usedBytes.low >>> 0, message.usedBytes.high >>> 0).toNumber(true) : message.usedBytes;
                if (message.entries != null && Object.hasOwnProperty.call(message, "entries"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.entries = typeof message.entries === "number" ? BigInt(message.entries) : $util.Long.fromBits(message.entries.low >>> 0, message.entries.high >>> 0, true).toBigInt();
                    else if (typeof message.entries === "number")
                        object.entries = options.longs === String ? String(message.entries) : message.entries;
                    else
                        object.entries = options.longs === String ? $util.Long.prototype.toString.call(message.entries) : options.longs === Number ? new $util.LongBits(message.entries.low >>> 0, message.entries.high >>> 0).toNumber(true) : message.entries;
                if (message.hits != null && Object.hasOwnProperty.call(message, "hits"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.hits = typeof message.hits === "number" ? BigInt(message.hits) : $util.Long.fromBits(message.hits.low >>> 0, message.hits.high >>> 0, true).toBigInt();
                    else if (typeof message.hits === "number")
                        object.hits = options.longs === String ? String(message.hits) : message.hits;
                    else
                        object.hits = options.longs === String ? $util.Long.prototype.toString.call(message.hits) : options.longs === Number ? new $util.LongBits(message.hits.low >>> 0, message.hits.high >>> 0).toNumber(true) : message.hits;
                if (message.misses != null && Object.hasOwnProperty.call(message, "misses"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.misses = typeof message.misses === "number" ? BigInt(message.misses) : $util.Long.fromBits(message.misses.low >>> 0, message.misses.high >>> 0, true).toBigInt();
                    else if (typeof message.misses === "number")
                        object.misses = options.longs === String ? String(message.misses) : message.misses;
                    else
                        object.misses = options.longs === String ? $util.Long.prototype.toString.call(message.misses) : options.longs === Number ? new $util.LongBits(message.misses.low >>> 0, message.misses.high >>> 0).toNumber(true) : message.misses;
                return object;
            };

            /**
             * Converts this CacheStats to JSON.
             * @function toJSON
             * @memberof revault.bindings.CacheStats
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            CacheStats.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for CacheStats
             * @function getTypeUrl
             * @memberof revault.bindings.CacheStats
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            CacheStats.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.CacheStats";
            };

            return CacheStats;
        })();

        bindings.ImportStats = (function() {

            /**
             * Properties of an ImportStats.
             * @memberof revault.bindings
             * @interface IImportStats
             * @property {string|null} [hostStatNanos] ImportStats hostStatNanos
             * @property {string|null} [hostReadNanos] ImportStats hostReadNanos
             * @property {string|null} [framePrepareNanos] ImportStats framePrepareNanos
             * @property {string|null} [pageWriteNanos] ImportStats pageWriteNanos
             */

            /**
             * Constructs a new ImportStats.
             * @memberof revault.bindings
             * @classdesc Represents an ImportStats.
             * @implements IImportStats
             * @constructor
             * @param {revault.bindings.IImportStats=} [properties] Properties to set
             */
            function ImportStats(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ImportStats hostStatNanos.
             * @member {string} hostStatNanos
             * @memberof revault.bindings.ImportStats
             * @instance
             */
            ImportStats.prototype.hostStatNanos = "";

            /**
             * ImportStats hostReadNanos.
             * @member {string} hostReadNanos
             * @memberof revault.bindings.ImportStats
             * @instance
             */
            ImportStats.prototype.hostReadNanos = "";

            /**
             * ImportStats framePrepareNanos.
             * @member {string} framePrepareNanos
             * @memberof revault.bindings.ImportStats
             * @instance
             */
            ImportStats.prototype.framePrepareNanos = "";

            /**
             * ImportStats pageWriteNanos.
             * @member {string} pageWriteNanos
             * @memberof revault.bindings.ImportStats
             * @instance
             */
            ImportStats.prototype.pageWriteNanos = "";

            /**
             * Creates a new ImportStats instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {revault.bindings.IImportStats=} [properties] Properties to set
             * @returns {revault.bindings.ImportStats} ImportStats instance
             */
            ImportStats.create = function create(properties) {
                return new ImportStats(properties);
            };

            /**
             * Encodes the specified ImportStats message. Does not implicitly {@link revault.bindings.ImportStats.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {revault.bindings.IImportStats} message ImportStats message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ImportStats.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.hostStatNanos != null && Object.hasOwnProperty.call(message, "hostStatNanos"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.hostStatNanos);
                if (message.hostReadNanos != null && Object.hasOwnProperty.call(message, "hostReadNanos"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.hostReadNanos);
                if (message.framePrepareNanos != null && Object.hasOwnProperty.call(message, "framePrepareNanos"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.framePrepareNanos);
                if (message.pageWriteNanos != null && Object.hasOwnProperty.call(message, "pageWriteNanos"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.pageWriteNanos);
                return writer;
            };

            /**
             * Encodes the specified ImportStats message, length delimited. Does not implicitly {@link revault.bindings.ImportStats.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {revault.bindings.IImportStats} message ImportStats message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ImportStats.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an ImportStats message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ImportStats} ImportStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ImportStats.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ImportStats();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.hostStatNanos = reader.string();
                            break;
                        }
                    case 2: {
                            message.hostReadNanos = reader.string();
                            break;
                        }
                    case 3: {
                            message.framePrepareNanos = reader.string();
                            break;
                        }
                    case 4: {
                            message.pageWriteNanos = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an ImportStats message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ImportStats} ImportStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ImportStats.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an ImportStats message.
             * @function verify
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ImportStats.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.hostStatNanos != null && Object.hasOwnProperty.call(message, "hostStatNanos"))
                    if (!$util.isString(message.hostStatNanos))
                        return "hostStatNanos: string expected";
                if (message.hostReadNanos != null && Object.hasOwnProperty.call(message, "hostReadNanos"))
                    if (!$util.isString(message.hostReadNanos))
                        return "hostReadNanos: string expected";
                if (message.framePrepareNanos != null && Object.hasOwnProperty.call(message, "framePrepareNanos"))
                    if (!$util.isString(message.framePrepareNanos))
                        return "framePrepareNanos: string expected";
                if (message.pageWriteNanos != null && Object.hasOwnProperty.call(message, "pageWriteNanos"))
                    if (!$util.isString(message.pageWriteNanos))
                        return "pageWriteNanos: string expected";
                return null;
            };

            /**
             * Creates an ImportStats message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ImportStats} ImportStats
             */
            ImportStats.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ImportStats)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ImportStats: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ImportStats();
                if (object.hostStatNanos != null)
                    message.hostStatNanos = String(object.hostStatNanos);
                if (object.hostReadNanos != null)
                    message.hostReadNanos = String(object.hostReadNanos);
                if (object.framePrepareNanos != null)
                    message.framePrepareNanos = String(object.framePrepareNanos);
                if (object.pageWriteNanos != null)
                    message.pageWriteNanos = String(object.pageWriteNanos);
                return message;
            };

            /**
             * Creates a plain object from an ImportStats message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {revault.bindings.ImportStats} message ImportStats
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ImportStats.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.hostStatNanos = "";
                    object.hostReadNanos = "";
                    object.framePrepareNanos = "";
                    object.pageWriteNanos = "";
                }
                if (message.hostStatNanos != null && Object.hasOwnProperty.call(message, "hostStatNanos"))
                    object.hostStatNanos = message.hostStatNanos;
                if (message.hostReadNanos != null && Object.hasOwnProperty.call(message, "hostReadNanos"))
                    object.hostReadNanos = message.hostReadNanos;
                if (message.framePrepareNanos != null && Object.hasOwnProperty.call(message, "framePrepareNanos"))
                    object.framePrepareNanos = message.framePrepareNanos;
                if (message.pageWriteNanos != null && Object.hasOwnProperty.call(message, "pageWriteNanos"))
                    object.pageWriteNanos = message.pageWriteNanos;
                return object;
            };

            /**
             * Converts this ImportStats to JSON.
             * @function toJSON
             * @memberof revault.bindings.ImportStats
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ImportStats.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ImportStats
             * @function getTypeUrl
             * @memberof revault.bindings.ImportStats
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ImportStats.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ImportStats";
            };

            return ImportStats;
        })();

        bindings.PageObject = (function() {

            /**
             * Properties of a PageObject.
             * @memberof revault.bindings
             * @interface IPageObject
             * @property {number|Long|null} [id] PageObject id
             * @property {string|null} [kind] PageObject kind
             * @property {number|Long|null} [payloadLen] PageObject payloadLen
             */

            /**
             * Constructs a new PageObject.
             * @memberof revault.bindings
             * @classdesc Represents a PageObject.
             * @implements IPageObject
             * @constructor
             * @param {revault.bindings.IPageObject=} [properties] Properties to set
             */
            function PageObject(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PageObject id.
             * @member {number|Long} id
             * @memberof revault.bindings.PageObject
             * @instance
             */
            PageObject.prototype.id = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageObject kind.
             * @member {string} kind
             * @memberof revault.bindings.PageObject
             * @instance
             */
            PageObject.prototype.kind = "";

            /**
             * PageObject payloadLen.
             * @member {number|Long} payloadLen
             * @memberof revault.bindings.PageObject
             * @instance
             */
            PageObject.prototype.payloadLen = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new PageObject instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PageObject
             * @static
             * @param {revault.bindings.IPageObject=} [properties] Properties to set
             * @returns {revault.bindings.PageObject} PageObject instance
             */
            PageObject.create = function create(properties) {
                return new PageObject(properties);
            };

            /**
             * Encodes the specified PageObject message. Does not implicitly {@link revault.bindings.PageObject.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PageObject
             * @static
             * @param {revault.bindings.IPageObject} message PageObject message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageObject.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.id);
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.kind);
                if (message.payloadLen != null && Object.hasOwnProperty.call(message, "payloadLen"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.payloadLen);
                return writer;
            };

            /**
             * Encodes the specified PageObject message, length delimited. Does not implicitly {@link revault.bindings.PageObject.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PageObject
             * @static
             * @param {revault.bindings.IPageObject} message PageObject message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageObject.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PageObject message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PageObject
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PageObject} PageObject
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageObject.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PageObject();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.id = reader.uint64();
                            break;
                        }
                    case 2: {
                            message.kind = reader.string();
                            break;
                        }
                    case 3: {
                            message.payloadLen = reader.uint64();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PageObject message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PageObject
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PageObject} PageObject
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageObject.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PageObject message.
             * @function verify
             * @memberof revault.bindings.PageObject
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PageObject.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (!$util.isInteger(message.id) && !(message.id && $util.isInteger(message.id.low) && $util.isInteger(message.id.high)))
                        return "id: integer|Long expected";
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    if (!$util.isString(message.kind))
                        return "kind: string expected";
                if (message.payloadLen != null && Object.hasOwnProperty.call(message, "payloadLen"))
                    if (!$util.isInteger(message.payloadLen) && !(message.payloadLen && $util.isInteger(message.payloadLen.low) && $util.isInteger(message.payloadLen.high)))
                        return "payloadLen: integer|Long expected";
                return null;
            };

            /**
             * Creates a PageObject message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PageObject
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PageObject} PageObject
             */
            PageObject.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PageObject)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PageObject: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PageObject();
                if (object.id != null)
                    if ($util.Long)
                        message.id = $util.Long.fromValue(object.id, true);
                    else if (typeof object.id === "string")
                        message.id = parseInt(object.id, 10);
                    else if (typeof object.id === "number")
                        message.id = object.id;
                    else if (typeof object.id === "object")
                        message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber(true);
                if (object.kind != null)
                    message.kind = String(object.kind);
                if (object.payloadLen != null)
                    if ($util.Long)
                        message.payloadLen = $util.Long.fromValue(object.payloadLen, true);
                    else if (typeof object.payloadLen === "string")
                        message.payloadLen = parseInt(object.payloadLen, 10);
                    else if (typeof object.payloadLen === "number")
                        message.payloadLen = object.payloadLen;
                    else if (typeof object.payloadLen === "object")
                        message.payloadLen = new $util.LongBits(object.payloadLen.low >>> 0, object.payloadLen.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from a PageObject message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PageObject
             * @static
             * @param {revault.bindings.PageObject} message PageObject
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PageObject.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.id = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.kind = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.payloadLen = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.payloadLen = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.id = typeof message.id === "number" ? BigInt(message.id) : $util.Long.fromBits(message.id.low >>> 0, message.id.high >>> 0, true).toBigInt();
                    else if (typeof message.id === "number")
                        object.id = options.longs === String ? String(message.id) : message.id;
                    else
                        object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber(true) : message.id;
                if (message.kind != null && Object.hasOwnProperty.call(message, "kind"))
                    object.kind = message.kind;
                if (message.payloadLen != null && Object.hasOwnProperty.call(message, "payloadLen"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.payloadLen = typeof message.payloadLen === "number" ? BigInt(message.payloadLen) : $util.Long.fromBits(message.payloadLen.low >>> 0, message.payloadLen.high >>> 0, true).toBigInt();
                    else if (typeof message.payloadLen === "number")
                        object.payloadLen = options.longs === String ? String(message.payloadLen) : message.payloadLen;
                    else
                        object.payloadLen = options.longs === String ? $util.Long.prototype.toString.call(message.payloadLen) : options.longs === Number ? new $util.LongBits(message.payloadLen.low >>> 0, message.payloadLen.high >>> 0).toNumber(true) : message.payloadLen;
                return object;
            };

            /**
             * Converts this PageObject to JSON.
             * @function toJSON
             * @memberof revault.bindings.PageObject
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PageObject.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PageObject
             * @function getTypeUrl
             * @memberof revault.bindings.PageObject
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PageObject.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PageObject";
            };

            return PageObject;
        })();

        bindings.PageInspection = (function() {

            /**
             * Properties of a PageInspection.
             * @memberof revault.bindings
             * @interface IPageInspection
             * @property {number|Long|null} [offset] PageInspection offset
             * @property {number|Long|null} [pageId] PageInspection pageId
             * @property {number|Long|null} [sequence] PageInspection sequence
             * @property {number|Long|null} [pageSize] PageInspection pageSize
             * @property {number|Long|null} [encryptedBodyLen] PageInspection encryptedBodyLen
             * @property {number|Long|null} [unusedBytes] PageInspection unusedBytes
             * @property {number|Long|null} [objectCount] PageInspection objectCount
             * @property {Array.<revault.bindings.IPageObject>|null} [objects] PageInspection objects
             */

            /**
             * Constructs a new PageInspection.
             * @memberof revault.bindings
             * @classdesc Represents a PageInspection.
             * @implements IPageInspection
             * @constructor
             * @param {revault.bindings.IPageInspection=} [properties] Properties to set
             */
            function PageInspection(properties) {
                this.objects = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PageInspection offset.
             * @member {number|Long} offset
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.offset = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection pageId.
             * @member {number|Long} pageId
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.pageId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection sequence.
             * @member {number|Long} sequence
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.sequence = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection pageSize.
             * @member {number|Long} pageSize
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.pageSize = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection encryptedBodyLen.
             * @member {number|Long} encryptedBodyLen
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.encryptedBodyLen = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection unusedBytes.
             * @member {number|Long} unusedBytes
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.unusedBytes = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection objectCount.
             * @member {number|Long} objectCount
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.objectCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * PageInspection objects.
             * @member {Array.<revault.bindings.IPageObject>} objects
             * @memberof revault.bindings.PageInspection
             * @instance
             */
            PageInspection.prototype.objects = $util.emptyArray;

            /**
             * Creates a new PageInspection instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {revault.bindings.IPageInspection=} [properties] Properties to set
             * @returns {revault.bindings.PageInspection} PageInspection instance
             */
            PageInspection.create = function create(properties) {
                return new PageInspection(properties);
            };

            /**
             * Encodes the specified PageInspection message. Does not implicitly {@link revault.bindings.PageInspection.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {revault.bindings.IPageInspection} message PageInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageInspection.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.offset != null && Object.hasOwnProperty.call(message, "offset"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.offset);
                if (message.pageId != null && Object.hasOwnProperty.call(message, "pageId"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.pageId);
                if (message.sequence != null && Object.hasOwnProperty.call(message, "sequence"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.sequence);
                if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.pageSize);
                if (message.encryptedBodyLen != null && Object.hasOwnProperty.call(message, "encryptedBodyLen"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.encryptedBodyLen);
                if (message.unusedBytes != null && Object.hasOwnProperty.call(message, "unusedBytes"))
                    writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.unusedBytes);
                if (message.objectCount != null && Object.hasOwnProperty.call(message, "objectCount"))
                    writer.uint32(/* id 7, wireType 0 =*/56).uint64(message.objectCount);
                if (message.objects != null && message.objects.length)
                    for (let i = 0; i < message.objects.length; ++i)
                        $root.revault.bindings.PageObject.encode(message.objects[i], writer.uint32(/* id 8, wireType 2 =*/66).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified PageInspection message, length delimited. Does not implicitly {@link revault.bindings.PageInspection.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {revault.bindings.IPageInspection} message PageInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageInspection.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PageInspection message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PageInspection} PageInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageInspection.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PageInspection();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.offset = reader.uint64();
                            break;
                        }
                    case 2: {
                            message.pageId = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.sequence = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.pageSize = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.encryptedBodyLen = reader.uint64();
                            break;
                        }
                    case 6: {
                            message.unusedBytes = reader.uint64();
                            break;
                        }
                    case 7: {
                            message.objectCount = reader.uint64();
                            break;
                        }
                    case 8: {
                            if (!(message.objects && message.objects.length))
                                message.objects = [];
                            message.objects.push($root.revault.bindings.PageObject.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PageInspection message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PageInspection} PageInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageInspection.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PageInspection message.
             * @function verify
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PageInspection.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.offset != null && Object.hasOwnProperty.call(message, "offset"))
                    if (!$util.isInteger(message.offset) && !(message.offset && $util.isInteger(message.offset.low) && $util.isInteger(message.offset.high)))
                        return "offset: integer|Long expected";
                if (message.pageId != null && Object.hasOwnProperty.call(message, "pageId"))
                    if (!$util.isInteger(message.pageId) && !(message.pageId && $util.isInteger(message.pageId.low) && $util.isInteger(message.pageId.high)))
                        return "pageId: integer|Long expected";
                if (message.sequence != null && Object.hasOwnProperty.call(message, "sequence"))
                    if (!$util.isInteger(message.sequence) && !(message.sequence && $util.isInteger(message.sequence.low) && $util.isInteger(message.sequence.high)))
                        return "sequence: integer|Long expected";
                if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                    if (!$util.isInteger(message.pageSize) && !(message.pageSize && $util.isInteger(message.pageSize.low) && $util.isInteger(message.pageSize.high)))
                        return "pageSize: integer|Long expected";
                if (message.encryptedBodyLen != null && Object.hasOwnProperty.call(message, "encryptedBodyLen"))
                    if (!$util.isInteger(message.encryptedBodyLen) && !(message.encryptedBodyLen && $util.isInteger(message.encryptedBodyLen.low) && $util.isInteger(message.encryptedBodyLen.high)))
                        return "encryptedBodyLen: integer|Long expected";
                if (message.unusedBytes != null && Object.hasOwnProperty.call(message, "unusedBytes"))
                    if (!$util.isInteger(message.unusedBytes) && !(message.unusedBytes && $util.isInteger(message.unusedBytes.low) && $util.isInteger(message.unusedBytes.high)))
                        return "unusedBytes: integer|Long expected";
                if (message.objectCount != null && Object.hasOwnProperty.call(message, "objectCount"))
                    if (!$util.isInteger(message.objectCount) && !(message.objectCount && $util.isInteger(message.objectCount.low) && $util.isInteger(message.objectCount.high)))
                        return "objectCount: integer|Long expected";
                if (message.objects != null && Object.hasOwnProperty.call(message, "objects")) {
                    if (!Array.isArray(message.objects))
                        return "objects: array expected";
                    for (let i = 0; i < message.objects.length; ++i) {
                        let error = $root.revault.bindings.PageObject.verify(message.objects[i], long + 1);
                        if (error)
                            return "objects." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a PageInspection message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PageInspection} PageInspection
             */
            PageInspection.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PageInspection)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PageInspection: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PageInspection();
                if (object.offset != null)
                    if ($util.Long)
                        message.offset = $util.Long.fromValue(object.offset, true);
                    else if (typeof object.offset === "string")
                        message.offset = parseInt(object.offset, 10);
                    else if (typeof object.offset === "number")
                        message.offset = object.offset;
                    else if (typeof object.offset === "object")
                        message.offset = new $util.LongBits(object.offset.low >>> 0, object.offset.high >>> 0).toNumber(true);
                if (object.pageId != null)
                    if ($util.Long)
                        message.pageId = $util.Long.fromValue(object.pageId, true);
                    else if (typeof object.pageId === "string")
                        message.pageId = parseInt(object.pageId, 10);
                    else if (typeof object.pageId === "number")
                        message.pageId = object.pageId;
                    else if (typeof object.pageId === "object")
                        message.pageId = new $util.LongBits(object.pageId.low >>> 0, object.pageId.high >>> 0).toNumber(true);
                if (object.sequence != null)
                    if ($util.Long)
                        message.sequence = $util.Long.fromValue(object.sequence, true);
                    else if (typeof object.sequence === "string")
                        message.sequence = parseInt(object.sequence, 10);
                    else if (typeof object.sequence === "number")
                        message.sequence = object.sequence;
                    else if (typeof object.sequence === "object")
                        message.sequence = new $util.LongBits(object.sequence.low >>> 0, object.sequence.high >>> 0).toNumber(true);
                if (object.pageSize != null)
                    if ($util.Long)
                        message.pageSize = $util.Long.fromValue(object.pageSize, true);
                    else if (typeof object.pageSize === "string")
                        message.pageSize = parseInt(object.pageSize, 10);
                    else if (typeof object.pageSize === "number")
                        message.pageSize = object.pageSize;
                    else if (typeof object.pageSize === "object")
                        message.pageSize = new $util.LongBits(object.pageSize.low >>> 0, object.pageSize.high >>> 0).toNumber(true);
                if (object.encryptedBodyLen != null)
                    if ($util.Long)
                        message.encryptedBodyLen = $util.Long.fromValue(object.encryptedBodyLen, true);
                    else if (typeof object.encryptedBodyLen === "string")
                        message.encryptedBodyLen = parseInt(object.encryptedBodyLen, 10);
                    else if (typeof object.encryptedBodyLen === "number")
                        message.encryptedBodyLen = object.encryptedBodyLen;
                    else if (typeof object.encryptedBodyLen === "object")
                        message.encryptedBodyLen = new $util.LongBits(object.encryptedBodyLen.low >>> 0, object.encryptedBodyLen.high >>> 0).toNumber(true);
                if (object.unusedBytes != null)
                    if ($util.Long)
                        message.unusedBytes = $util.Long.fromValue(object.unusedBytes, true);
                    else if (typeof object.unusedBytes === "string")
                        message.unusedBytes = parseInt(object.unusedBytes, 10);
                    else if (typeof object.unusedBytes === "number")
                        message.unusedBytes = object.unusedBytes;
                    else if (typeof object.unusedBytes === "object")
                        message.unusedBytes = new $util.LongBits(object.unusedBytes.low >>> 0, object.unusedBytes.high >>> 0).toNumber(true);
                if (object.objectCount != null)
                    if ($util.Long)
                        message.objectCount = $util.Long.fromValue(object.objectCount, true);
                    else if (typeof object.objectCount === "string")
                        message.objectCount = parseInt(object.objectCount, 10);
                    else if (typeof object.objectCount === "number")
                        message.objectCount = object.objectCount;
                    else if (typeof object.objectCount === "object")
                        message.objectCount = new $util.LongBits(object.objectCount.low >>> 0, object.objectCount.high >>> 0).toNumber(true);
                if (object.objects) {
                    if (!Array.isArray(object.objects))
                        throw TypeError(".revault.bindings.PageInspection.objects: array expected");
                    message.objects = [];
                    for (let i = 0; i < object.objects.length; ++i) {
                        if (!$util.isObject(object.objects[i]))
                            throw TypeError(".revault.bindings.PageInspection.objects: object expected");
                        message.objects[i] = $root.revault.bindings.PageObject.fromObject(object.objects[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a PageInspection message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {revault.bindings.PageInspection} message PageInspection
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PageInspection.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.objects = [];
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.offset = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.offset = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.pageId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.pageId = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.sequence = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.sequence = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.pageSize = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.pageSize = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.encryptedBodyLen = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.encryptedBodyLen = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.unusedBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.unusedBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.objectCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.objectCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.offset != null && Object.hasOwnProperty.call(message, "offset"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.offset = typeof message.offset === "number" ? BigInt(message.offset) : $util.Long.fromBits(message.offset.low >>> 0, message.offset.high >>> 0, true).toBigInt();
                    else if (typeof message.offset === "number")
                        object.offset = options.longs === String ? String(message.offset) : message.offset;
                    else
                        object.offset = options.longs === String ? $util.Long.prototype.toString.call(message.offset) : options.longs === Number ? new $util.LongBits(message.offset.low >>> 0, message.offset.high >>> 0).toNumber(true) : message.offset;
                if (message.pageId != null && Object.hasOwnProperty.call(message, "pageId"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.pageId = typeof message.pageId === "number" ? BigInt(message.pageId) : $util.Long.fromBits(message.pageId.low >>> 0, message.pageId.high >>> 0, true).toBigInt();
                    else if (typeof message.pageId === "number")
                        object.pageId = options.longs === String ? String(message.pageId) : message.pageId;
                    else
                        object.pageId = options.longs === String ? $util.Long.prototype.toString.call(message.pageId) : options.longs === Number ? new $util.LongBits(message.pageId.low >>> 0, message.pageId.high >>> 0).toNumber(true) : message.pageId;
                if (message.sequence != null && Object.hasOwnProperty.call(message, "sequence"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.sequence = typeof message.sequence === "number" ? BigInt(message.sequence) : $util.Long.fromBits(message.sequence.low >>> 0, message.sequence.high >>> 0, true).toBigInt();
                    else if (typeof message.sequence === "number")
                        object.sequence = options.longs === String ? String(message.sequence) : message.sequence;
                    else
                        object.sequence = options.longs === String ? $util.Long.prototype.toString.call(message.sequence) : options.longs === Number ? new $util.LongBits(message.sequence.low >>> 0, message.sequence.high >>> 0).toNumber(true) : message.sequence;
                if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.pageSize = typeof message.pageSize === "number" ? BigInt(message.pageSize) : $util.Long.fromBits(message.pageSize.low >>> 0, message.pageSize.high >>> 0, true).toBigInt();
                    else if (typeof message.pageSize === "number")
                        object.pageSize = options.longs === String ? String(message.pageSize) : message.pageSize;
                    else
                        object.pageSize = options.longs === String ? $util.Long.prototype.toString.call(message.pageSize) : options.longs === Number ? new $util.LongBits(message.pageSize.low >>> 0, message.pageSize.high >>> 0).toNumber(true) : message.pageSize;
                if (message.encryptedBodyLen != null && Object.hasOwnProperty.call(message, "encryptedBodyLen"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.encryptedBodyLen = typeof message.encryptedBodyLen === "number" ? BigInt(message.encryptedBodyLen) : $util.Long.fromBits(message.encryptedBodyLen.low >>> 0, message.encryptedBodyLen.high >>> 0, true).toBigInt();
                    else if (typeof message.encryptedBodyLen === "number")
                        object.encryptedBodyLen = options.longs === String ? String(message.encryptedBodyLen) : message.encryptedBodyLen;
                    else
                        object.encryptedBodyLen = options.longs === String ? $util.Long.prototype.toString.call(message.encryptedBodyLen) : options.longs === Number ? new $util.LongBits(message.encryptedBodyLen.low >>> 0, message.encryptedBodyLen.high >>> 0).toNumber(true) : message.encryptedBodyLen;
                if (message.unusedBytes != null && Object.hasOwnProperty.call(message, "unusedBytes"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.unusedBytes = typeof message.unusedBytes === "number" ? BigInt(message.unusedBytes) : $util.Long.fromBits(message.unusedBytes.low >>> 0, message.unusedBytes.high >>> 0, true).toBigInt();
                    else if (typeof message.unusedBytes === "number")
                        object.unusedBytes = options.longs === String ? String(message.unusedBytes) : message.unusedBytes;
                    else
                        object.unusedBytes = options.longs === String ? $util.Long.prototype.toString.call(message.unusedBytes) : options.longs === Number ? new $util.LongBits(message.unusedBytes.low >>> 0, message.unusedBytes.high >>> 0).toNumber(true) : message.unusedBytes;
                if (message.objectCount != null && Object.hasOwnProperty.call(message, "objectCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.objectCount = typeof message.objectCount === "number" ? BigInt(message.objectCount) : $util.Long.fromBits(message.objectCount.low >>> 0, message.objectCount.high >>> 0, true).toBigInt();
                    else if (typeof message.objectCount === "number")
                        object.objectCount = options.longs === String ? String(message.objectCount) : message.objectCount;
                    else
                        object.objectCount = options.longs === String ? $util.Long.prototype.toString.call(message.objectCount) : options.longs === Number ? new $util.LongBits(message.objectCount.low >>> 0, message.objectCount.high >>> 0).toNumber(true) : message.objectCount;
                if (message.objects && message.objects.length) {
                    object.objects = [];
                    for (let j = 0; j < message.objects.length; ++j)
                        object.objects[j] = $root.revault.bindings.PageObject.toObject(message.objects[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this PageInspection to JSON.
             * @function toJSON
             * @memberof revault.bindings.PageInspection
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PageInspection.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PageInspection
             * @function getTypeUrl
             * @memberof revault.bindings.PageInspection
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PageInspection.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PageInspection";
            };

            return PageInspection;
        })();

        bindings.PageInspectionList = (function() {

            /**
             * Properties of a PageInspectionList.
             * @memberof revault.bindings
             * @interface IPageInspectionList
             * @property {Array.<revault.bindings.IPageInspection>|null} [values] PageInspectionList values
             */

            /**
             * Constructs a new PageInspectionList.
             * @memberof revault.bindings
             * @classdesc Represents a PageInspectionList.
             * @implements IPageInspectionList
             * @constructor
             * @param {revault.bindings.IPageInspectionList=} [properties] Properties to set
             */
            function PageInspectionList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PageInspectionList values.
             * @member {Array.<revault.bindings.IPageInspection>} values
             * @memberof revault.bindings.PageInspectionList
             * @instance
             */
            PageInspectionList.prototype.values = $util.emptyArray;

            /**
             * Creates a new PageInspectionList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {revault.bindings.IPageInspectionList=} [properties] Properties to set
             * @returns {revault.bindings.PageInspectionList} PageInspectionList instance
             */
            PageInspectionList.create = function create(properties) {
                return new PageInspectionList(properties);
            };

            /**
             * Encodes the specified PageInspectionList message. Does not implicitly {@link revault.bindings.PageInspectionList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {revault.bindings.IPageInspectionList} message PageInspectionList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageInspectionList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.PageInspection.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified PageInspectionList message, length delimited. Does not implicitly {@link revault.bindings.PageInspectionList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {revault.bindings.IPageInspectionList} message PageInspectionList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PageInspectionList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PageInspectionList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PageInspectionList} PageInspectionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageInspectionList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PageInspectionList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.PageInspection.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PageInspectionList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PageInspectionList} PageInspectionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PageInspectionList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PageInspectionList message.
             * @function verify
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PageInspectionList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.PageInspection.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a PageInspectionList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PageInspectionList} PageInspectionList
             */
            PageInspectionList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PageInspectionList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PageInspectionList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PageInspectionList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.PageInspectionList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.PageInspectionList.values: object expected");
                        message.values[i] = $root.revault.bindings.PageInspection.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a PageInspectionList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {revault.bindings.PageInspectionList} message PageInspectionList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PageInspectionList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.PageInspection.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this PageInspectionList to JSON.
             * @function toJSON
             * @memberof revault.bindings.PageInspectionList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PageInspectionList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PageInspectionList
             * @function getTypeUrl
             * @memberof revault.bindings.PageInspectionList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PageInspectionList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PageInspectionList";
            };

            return PageInspectionList;
        })();

        bindings.FileInspection = (function() {

            /**
             * Properties of a FileInspection.
             * @memberof revault.bindings
             * @interface IFileInspection
             * @property {Uint8Array|null} [lockboxId] FileInspection lockboxId
             * @property {boolean|null} [headerReadable] FileInspection headerReadable
             * @property {number|Long|null} [keyDirectoryGeneration] FileInspection keyDirectoryGeneration
             * @property {number|Long|null} [keyDirectoryCopyCount] FileInspection keyDirectoryCopyCount
             * @property {boolean|null} [ownerSigned] FileInspection ownerSigned
             * @property {Array.<revault.bindings.IKeySlot>|null} [keySlots] FileInspection keySlots
             */

            /**
             * Constructs a new FileInspection.
             * @memberof revault.bindings
             * @classdesc Represents a FileInspection.
             * @implements IFileInspection
             * @constructor
             * @param {revault.bindings.IFileInspection=} [properties] Properties to set
             */
            function FileInspection(properties) {
                this.keySlots = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FileInspection lockboxId.
             * @member {Uint8Array} lockboxId
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.lockboxId = $util.newBuffer([]);

            /**
             * FileInspection headerReadable.
             * @member {boolean} headerReadable
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.headerReadable = false;

            /**
             * FileInspection keyDirectoryGeneration.
             * @member {number|Long} keyDirectoryGeneration
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.keyDirectoryGeneration = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * FileInspection keyDirectoryCopyCount.
             * @member {number|Long} keyDirectoryCopyCount
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.keyDirectoryCopyCount = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * FileInspection ownerSigned.
             * @member {boolean} ownerSigned
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.ownerSigned = false;

            /**
             * FileInspection keySlots.
             * @member {Array.<revault.bindings.IKeySlot>} keySlots
             * @memberof revault.bindings.FileInspection
             * @instance
             */
            FileInspection.prototype.keySlots = $util.emptyArray;

            /**
             * Creates a new FileInspection instance using the specified properties.
             * @function create
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {revault.bindings.IFileInspection=} [properties] Properties to set
             * @returns {revault.bindings.FileInspection} FileInspection instance
             */
            FileInspection.create = function create(properties) {
                return new FileInspection(properties);
            };

            /**
             * Encodes the specified FileInspection message. Does not implicitly {@link revault.bindings.FileInspection.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {revault.bindings.IFileInspection} message FileInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileInspection.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.lockboxId);
                if (message.headerReadable != null && Object.hasOwnProperty.call(message, "headerReadable"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.headerReadable);
                if (message.keyDirectoryGeneration != null && Object.hasOwnProperty.call(message, "keyDirectoryGeneration"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.keyDirectoryGeneration);
                if (message.keyDirectoryCopyCount != null && Object.hasOwnProperty.call(message, "keyDirectoryCopyCount"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.keyDirectoryCopyCount);
                if (message.ownerSigned != null && Object.hasOwnProperty.call(message, "ownerSigned"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.ownerSigned);
                if (message.keySlots != null && message.keySlots.length)
                    for (let i = 0; i < message.keySlots.length; ++i)
                        $root.revault.bindings.KeySlot.encode(message.keySlots[i], writer.uint32(/* id 6, wireType 2 =*/50).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FileInspection message, length delimited. Does not implicitly {@link revault.bindings.FileInspection.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {revault.bindings.IFileInspection} message FileInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileInspection.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FileInspection message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.FileInspection} FileInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileInspection.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.FileInspection();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.lockboxId = reader.bytes();
                            break;
                        }
                    case 2: {
                            message.headerReadable = reader.bool();
                            break;
                        }
                    case 3: {
                            message.keyDirectoryGeneration = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.keyDirectoryCopyCount = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.ownerSigned = reader.bool();
                            break;
                        }
                    case 6: {
                            if (!(message.keySlots && message.keySlots.length))
                                message.keySlots = [];
                            message.keySlots.push($root.revault.bindings.KeySlot.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FileInspection message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.FileInspection} FileInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileInspection.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FileInspection message.
             * @function verify
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FileInspection.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    if (!(message.lockboxId && typeof message.lockboxId.length === "number" || $util.isString(message.lockboxId)))
                        return "lockboxId: buffer expected";
                if (message.headerReadable != null && Object.hasOwnProperty.call(message, "headerReadable"))
                    if (typeof message.headerReadable !== "boolean")
                        return "headerReadable: boolean expected";
                if (message.keyDirectoryGeneration != null && Object.hasOwnProperty.call(message, "keyDirectoryGeneration"))
                    if (!$util.isInteger(message.keyDirectoryGeneration) && !(message.keyDirectoryGeneration && $util.isInteger(message.keyDirectoryGeneration.low) && $util.isInteger(message.keyDirectoryGeneration.high)))
                        return "keyDirectoryGeneration: integer|Long expected";
                if (message.keyDirectoryCopyCount != null && Object.hasOwnProperty.call(message, "keyDirectoryCopyCount"))
                    if (!$util.isInteger(message.keyDirectoryCopyCount) && !(message.keyDirectoryCopyCount && $util.isInteger(message.keyDirectoryCopyCount.low) && $util.isInteger(message.keyDirectoryCopyCount.high)))
                        return "keyDirectoryCopyCount: integer|Long expected";
                if (message.ownerSigned != null && Object.hasOwnProperty.call(message, "ownerSigned"))
                    if (typeof message.ownerSigned !== "boolean")
                        return "ownerSigned: boolean expected";
                if (message.keySlots != null && Object.hasOwnProperty.call(message, "keySlots")) {
                    if (!Array.isArray(message.keySlots))
                        return "keySlots: array expected";
                    for (let i = 0; i < message.keySlots.length; ++i) {
                        let error = $root.revault.bindings.KeySlot.verify(message.keySlots[i], long + 1);
                        if (error)
                            return "keySlots." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FileInspection message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.FileInspection} FileInspection
             */
            FileInspection.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.FileInspection)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.FileInspection: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.FileInspection();
                if (object.lockboxId != null)
                    if (typeof object.lockboxId === "string")
                        $util.base64.decode(object.lockboxId, message.lockboxId = $util.newBuffer($util.base64.length(object.lockboxId)), 0);
                    else if (object.lockboxId.length >= 0)
                        message.lockboxId = object.lockboxId;
                if (object.headerReadable != null)
                    message.headerReadable = Boolean(object.headerReadable);
                if (object.keyDirectoryGeneration != null)
                    if ($util.Long)
                        message.keyDirectoryGeneration = $util.Long.fromValue(object.keyDirectoryGeneration, true);
                    else if (typeof object.keyDirectoryGeneration === "string")
                        message.keyDirectoryGeneration = parseInt(object.keyDirectoryGeneration, 10);
                    else if (typeof object.keyDirectoryGeneration === "number")
                        message.keyDirectoryGeneration = object.keyDirectoryGeneration;
                    else if (typeof object.keyDirectoryGeneration === "object")
                        message.keyDirectoryGeneration = new $util.LongBits(object.keyDirectoryGeneration.low >>> 0, object.keyDirectoryGeneration.high >>> 0).toNumber(true);
                if (object.keyDirectoryCopyCount != null)
                    if ($util.Long)
                        message.keyDirectoryCopyCount = $util.Long.fromValue(object.keyDirectoryCopyCount, true);
                    else if (typeof object.keyDirectoryCopyCount === "string")
                        message.keyDirectoryCopyCount = parseInt(object.keyDirectoryCopyCount, 10);
                    else if (typeof object.keyDirectoryCopyCount === "number")
                        message.keyDirectoryCopyCount = object.keyDirectoryCopyCount;
                    else if (typeof object.keyDirectoryCopyCount === "object")
                        message.keyDirectoryCopyCount = new $util.LongBits(object.keyDirectoryCopyCount.low >>> 0, object.keyDirectoryCopyCount.high >>> 0).toNumber(true);
                if (object.ownerSigned != null)
                    message.ownerSigned = Boolean(object.ownerSigned);
                if (object.keySlots) {
                    if (!Array.isArray(object.keySlots))
                        throw TypeError(".revault.bindings.FileInspection.keySlots: array expected");
                    message.keySlots = [];
                    for (let i = 0; i < object.keySlots.length; ++i) {
                        if (!$util.isObject(object.keySlots[i]))
                            throw TypeError(".revault.bindings.FileInspection.keySlots: object expected");
                        message.keySlots[i] = $root.revault.bindings.KeySlot.fromObject(object.keySlots[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FileInspection message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {revault.bindings.FileInspection} message FileInspection
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FileInspection.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.keySlots = [];
                if (options.defaults) {
                    if (options.bytes === String)
                        object.lockboxId = "";
                    else {
                        object.lockboxId = [];
                        if (options.bytes !== Array)
                            object.lockboxId = $util.newBuffer(object.lockboxId);
                    }
                    object.headerReadable = false;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.keyDirectoryGeneration = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.keyDirectoryGeneration = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.keyDirectoryCopyCount = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.keyDirectoryCopyCount = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.ownerSigned = false;
                }
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    object.lockboxId = options.bytes === String ? $util.base64.encode(message.lockboxId, 0, message.lockboxId.length) : options.bytes === Array ? Array.prototype.slice.call(message.lockboxId) : message.lockboxId;
                if (message.headerReadable != null && Object.hasOwnProperty.call(message, "headerReadable"))
                    object.headerReadable = message.headerReadable;
                if (message.keyDirectoryGeneration != null && Object.hasOwnProperty.call(message, "keyDirectoryGeneration"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.keyDirectoryGeneration = typeof message.keyDirectoryGeneration === "number" ? BigInt(message.keyDirectoryGeneration) : $util.Long.fromBits(message.keyDirectoryGeneration.low >>> 0, message.keyDirectoryGeneration.high >>> 0, true).toBigInt();
                    else if (typeof message.keyDirectoryGeneration === "number")
                        object.keyDirectoryGeneration = options.longs === String ? String(message.keyDirectoryGeneration) : message.keyDirectoryGeneration;
                    else
                        object.keyDirectoryGeneration = options.longs === String ? $util.Long.prototype.toString.call(message.keyDirectoryGeneration) : options.longs === Number ? new $util.LongBits(message.keyDirectoryGeneration.low >>> 0, message.keyDirectoryGeneration.high >>> 0).toNumber(true) : message.keyDirectoryGeneration;
                if (message.keyDirectoryCopyCount != null && Object.hasOwnProperty.call(message, "keyDirectoryCopyCount"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.keyDirectoryCopyCount = typeof message.keyDirectoryCopyCount === "number" ? BigInt(message.keyDirectoryCopyCount) : $util.Long.fromBits(message.keyDirectoryCopyCount.low >>> 0, message.keyDirectoryCopyCount.high >>> 0, true).toBigInt();
                    else if (typeof message.keyDirectoryCopyCount === "number")
                        object.keyDirectoryCopyCount = options.longs === String ? String(message.keyDirectoryCopyCount) : message.keyDirectoryCopyCount;
                    else
                        object.keyDirectoryCopyCount = options.longs === String ? $util.Long.prototype.toString.call(message.keyDirectoryCopyCount) : options.longs === Number ? new $util.LongBits(message.keyDirectoryCopyCount.low >>> 0, message.keyDirectoryCopyCount.high >>> 0).toNumber(true) : message.keyDirectoryCopyCount;
                if (message.ownerSigned != null && Object.hasOwnProperty.call(message, "ownerSigned"))
                    object.ownerSigned = message.ownerSigned;
                if (message.keySlots && message.keySlots.length) {
                    object.keySlots = [];
                    for (let j = 0; j < message.keySlots.length; ++j)
                        object.keySlots[j] = $root.revault.bindings.KeySlot.toObject(message.keySlots[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this FileInspection to JSON.
             * @function toJSON
             * @memberof revault.bindings.FileInspection
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FileInspection.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for FileInspection
             * @function getTypeUrl
             * @memberof revault.bindings.FileInspection
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            FileInspection.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.FileInspection";
            };

            return FileInspection;
        })();

        bindings.ProfileGeneration = (function() {

            /**
             * Properties of a ProfileGeneration.
             * @memberof revault.bindings
             * @interface IProfileGeneration
             * @property {number|null} [index] ProfileGeneration index
             * @property {string|null} [status] ProfileGeneration status
             * @property {Uint8Array|null} [contactFingerprint] ProfileGeneration contactFingerprint
             * @property {number|Long|null} [createdAtUnixMs] ProfileGeneration createdAtUnixMs
             * @property {number|Long|null} [retiredAtUnixMs] ProfileGeneration retiredAtUnixMs
             * @property {boolean|null} [hasRetiredAt] ProfileGeneration hasRetiredAt
             */

            /**
             * Constructs a new ProfileGeneration.
             * @memberof revault.bindings
             * @classdesc Represents a ProfileGeneration.
             * @implements IProfileGeneration
             * @constructor
             * @param {revault.bindings.IProfileGeneration=} [properties] Properties to set
             */
            function ProfileGeneration(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ProfileGeneration index.
             * @member {number} index
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.index = 0;

            /**
             * ProfileGeneration status.
             * @member {string} status
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.status = "";

            /**
             * ProfileGeneration contactFingerprint.
             * @member {Uint8Array} contactFingerprint
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.contactFingerprint = $util.newBuffer([]);

            /**
             * ProfileGeneration createdAtUnixMs.
             * @member {number|Long} createdAtUnixMs
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.createdAtUnixMs = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * ProfileGeneration retiredAtUnixMs.
             * @member {number|Long} retiredAtUnixMs
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.retiredAtUnixMs = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * ProfileGeneration hasRetiredAt.
             * @member {boolean} hasRetiredAt
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             */
            ProfileGeneration.prototype.hasRetiredAt = false;

            /**
             * Creates a new ProfileGeneration instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {revault.bindings.IProfileGeneration=} [properties] Properties to set
             * @returns {revault.bindings.ProfileGeneration} ProfileGeneration instance
             */
            ProfileGeneration.create = function create(properties) {
                return new ProfileGeneration(properties);
            };

            /**
             * Encodes the specified ProfileGeneration message. Does not implicitly {@link revault.bindings.ProfileGeneration.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {revault.bindings.IProfileGeneration} message ProfileGeneration message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileGeneration.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.index != null && Object.hasOwnProperty.call(message, "index"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.index);
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.status);
                if (message.contactFingerprint != null && Object.hasOwnProperty.call(message, "contactFingerprint"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.contactFingerprint);
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.createdAtUnixMs);
                if (message.retiredAtUnixMs != null && Object.hasOwnProperty.call(message, "retiredAtUnixMs"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.retiredAtUnixMs);
                if (message.hasRetiredAt != null && Object.hasOwnProperty.call(message, "hasRetiredAt"))
                    writer.uint32(/* id 6, wireType 0 =*/48).bool(message.hasRetiredAt);
                return writer;
            };

            /**
             * Encodes the specified ProfileGeneration message, length delimited. Does not implicitly {@link revault.bindings.ProfileGeneration.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {revault.bindings.IProfileGeneration} message ProfileGeneration message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileGeneration.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ProfileGeneration message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ProfileGeneration} ProfileGeneration
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileGeneration.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ProfileGeneration();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.index = reader.uint32();
                            break;
                        }
                    case 2: {
                            message.status = reader.string();
                            break;
                        }
                    case 3: {
                            message.contactFingerprint = reader.bytes();
                            break;
                        }
                    case 4: {
                            message.createdAtUnixMs = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.retiredAtUnixMs = reader.uint64();
                            break;
                        }
                    case 6: {
                            message.hasRetiredAt = reader.bool();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ProfileGeneration message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ProfileGeneration} ProfileGeneration
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileGeneration.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ProfileGeneration message.
             * @function verify
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ProfileGeneration.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.index != null && Object.hasOwnProperty.call(message, "index"))
                    if (!$util.isInteger(message.index))
                        return "index: integer expected";
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    if (!$util.isString(message.status))
                        return "status: string expected";
                if (message.contactFingerprint != null && Object.hasOwnProperty.call(message, "contactFingerprint"))
                    if (!(message.contactFingerprint && typeof message.contactFingerprint.length === "number" || $util.isString(message.contactFingerprint)))
                        return "contactFingerprint: buffer expected";
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    if (!$util.isInteger(message.createdAtUnixMs) && !(message.createdAtUnixMs && $util.isInteger(message.createdAtUnixMs.low) && $util.isInteger(message.createdAtUnixMs.high)))
                        return "createdAtUnixMs: integer|Long expected";
                if (message.retiredAtUnixMs != null && Object.hasOwnProperty.call(message, "retiredAtUnixMs"))
                    if (!$util.isInteger(message.retiredAtUnixMs) && !(message.retiredAtUnixMs && $util.isInteger(message.retiredAtUnixMs.low) && $util.isInteger(message.retiredAtUnixMs.high)))
                        return "retiredAtUnixMs: integer|Long expected";
                if (message.hasRetiredAt != null && Object.hasOwnProperty.call(message, "hasRetiredAt"))
                    if (typeof message.hasRetiredAt !== "boolean")
                        return "hasRetiredAt: boolean expected";
                return null;
            };

            /**
             * Creates a ProfileGeneration message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ProfileGeneration} ProfileGeneration
             */
            ProfileGeneration.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ProfileGeneration)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ProfileGeneration: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ProfileGeneration();
                if (object.index != null)
                    message.index = object.index >>> 0;
                if (object.status != null)
                    message.status = String(object.status);
                if (object.contactFingerprint != null)
                    if (typeof object.contactFingerprint === "string")
                        $util.base64.decode(object.contactFingerprint, message.contactFingerprint = $util.newBuffer($util.base64.length(object.contactFingerprint)), 0);
                    else if (object.contactFingerprint.length >= 0)
                        message.contactFingerprint = object.contactFingerprint;
                if (object.createdAtUnixMs != null)
                    if ($util.Long)
                        message.createdAtUnixMs = $util.Long.fromValue(object.createdAtUnixMs, true);
                    else if (typeof object.createdAtUnixMs === "string")
                        message.createdAtUnixMs = parseInt(object.createdAtUnixMs, 10);
                    else if (typeof object.createdAtUnixMs === "number")
                        message.createdAtUnixMs = object.createdAtUnixMs;
                    else if (typeof object.createdAtUnixMs === "object")
                        message.createdAtUnixMs = new $util.LongBits(object.createdAtUnixMs.low >>> 0, object.createdAtUnixMs.high >>> 0).toNumber(true);
                if (object.retiredAtUnixMs != null)
                    if ($util.Long)
                        message.retiredAtUnixMs = $util.Long.fromValue(object.retiredAtUnixMs, true);
                    else if (typeof object.retiredAtUnixMs === "string")
                        message.retiredAtUnixMs = parseInt(object.retiredAtUnixMs, 10);
                    else if (typeof object.retiredAtUnixMs === "number")
                        message.retiredAtUnixMs = object.retiredAtUnixMs;
                    else if (typeof object.retiredAtUnixMs === "object")
                        message.retiredAtUnixMs = new $util.LongBits(object.retiredAtUnixMs.low >>> 0, object.retiredAtUnixMs.high >>> 0).toNumber(true);
                if (object.hasRetiredAt != null)
                    message.hasRetiredAt = Boolean(object.hasRetiredAt);
                return message;
            };

            /**
             * Creates a plain object from a ProfileGeneration message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {revault.bindings.ProfileGeneration} message ProfileGeneration
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ProfileGeneration.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.index = 0;
                    object.status = "";
                    if (options.bytes === String)
                        object.contactFingerprint = "";
                    else {
                        object.contactFingerprint = [];
                        if (options.bytes !== Array)
                            object.contactFingerprint = $util.newBuffer(object.contactFingerprint);
                    }
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.createdAtUnixMs = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.createdAtUnixMs = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.retiredAtUnixMs = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.retiredAtUnixMs = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.hasRetiredAt = false;
                }
                if (message.index != null && Object.hasOwnProperty.call(message, "index"))
                    object.index = message.index;
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    object.status = message.status;
                if (message.contactFingerprint != null && Object.hasOwnProperty.call(message, "contactFingerprint"))
                    object.contactFingerprint = options.bytes === String ? $util.base64.encode(message.contactFingerprint, 0, message.contactFingerprint.length) : options.bytes === Array ? Array.prototype.slice.call(message.contactFingerprint) : message.contactFingerprint;
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.createdAtUnixMs = typeof message.createdAtUnixMs === "number" ? BigInt(message.createdAtUnixMs) : $util.Long.fromBits(message.createdAtUnixMs.low >>> 0, message.createdAtUnixMs.high >>> 0, true).toBigInt();
                    else if (typeof message.createdAtUnixMs === "number")
                        object.createdAtUnixMs = options.longs === String ? String(message.createdAtUnixMs) : message.createdAtUnixMs;
                    else
                        object.createdAtUnixMs = options.longs === String ? $util.Long.prototype.toString.call(message.createdAtUnixMs) : options.longs === Number ? new $util.LongBits(message.createdAtUnixMs.low >>> 0, message.createdAtUnixMs.high >>> 0).toNumber(true) : message.createdAtUnixMs;
                if (message.retiredAtUnixMs != null && Object.hasOwnProperty.call(message, "retiredAtUnixMs"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.retiredAtUnixMs = typeof message.retiredAtUnixMs === "number" ? BigInt(message.retiredAtUnixMs) : $util.Long.fromBits(message.retiredAtUnixMs.low >>> 0, message.retiredAtUnixMs.high >>> 0, true).toBigInt();
                    else if (typeof message.retiredAtUnixMs === "number")
                        object.retiredAtUnixMs = options.longs === String ? String(message.retiredAtUnixMs) : message.retiredAtUnixMs;
                    else
                        object.retiredAtUnixMs = options.longs === String ? $util.Long.prototype.toString.call(message.retiredAtUnixMs) : options.longs === Number ? new $util.LongBits(message.retiredAtUnixMs.low >>> 0, message.retiredAtUnixMs.high >>> 0).toNumber(true) : message.retiredAtUnixMs;
                if (message.hasRetiredAt != null && Object.hasOwnProperty.call(message, "hasRetiredAt"))
                    object.hasRetiredAt = message.hasRetiredAt;
                return object;
            };

            /**
             * Converts this ProfileGeneration to JSON.
             * @function toJSON
             * @memberof revault.bindings.ProfileGeneration
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ProfileGeneration.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ProfileGeneration
             * @function getTypeUrl
             * @memberof revault.bindings.ProfileGeneration
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ProfileGeneration.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ProfileGeneration";
            };

            return ProfileGeneration;
        })();

        bindings.ProfileHistory = (function() {

            /**
             * Properties of a ProfileHistory.
             * @memberof revault.bindings
             * @interface IProfileHistory
             * @property {string|null} [name] ProfileHistory name
             * @property {number|null} [activeGeneration] ProfileHistory activeGeneration
             * @property {Array.<revault.bindings.IProfileGeneration>|null} [generations] ProfileHistory generations
             */

            /**
             * Constructs a new ProfileHistory.
             * @memberof revault.bindings
             * @classdesc Represents a ProfileHistory.
             * @implements IProfileHistory
             * @constructor
             * @param {revault.bindings.IProfileHistory=} [properties] Properties to set
             */
            function ProfileHistory(properties) {
                this.generations = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ProfileHistory name.
             * @member {string} name
             * @memberof revault.bindings.ProfileHistory
             * @instance
             */
            ProfileHistory.prototype.name = "";

            /**
             * ProfileHistory activeGeneration.
             * @member {number} activeGeneration
             * @memberof revault.bindings.ProfileHistory
             * @instance
             */
            ProfileHistory.prototype.activeGeneration = 0;

            /**
             * ProfileHistory generations.
             * @member {Array.<revault.bindings.IProfileGeneration>} generations
             * @memberof revault.bindings.ProfileHistory
             * @instance
             */
            ProfileHistory.prototype.generations = $util.emptyArray;

            /**
             * Creates a new ProfileHistory instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {revault.bindings.IProfileHistory=} [properties] Properties to set
             * @returns {revault.bindings.ProfileHistory} ProfileHistory instance
             */
            ProfileHistory.create = function create(properties) {
                return new ProfileHistory(properties);
            };

            /**
             * Encodes the specified ProfileHistory message. Does not implicitly {@link revault.bindings.ProfileHistory.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {revault.bindings.IProfileHistory} message ProfileHistory message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileHistory.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.activeGeneration != null && Object.hasOwnProperty.call(message, "activeGeneration"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.activeGeneration);
                if (message.generations != null && message.generations.length)
                    for (let i = 0; i < message.generations.length; ++i)
                        $root.revault.bindings.ProfileGeneration.encode(message.generations[i], writer.uint32(/* id 3, wireType 2 =*/26).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified ProfileHistory message, length delimited. Does not implicitly {@link revault.bindings.ProfileHistory.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {revault.bindings.IProfileHistory} message ProfileHistory message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileHistory.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ProfileHistory message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ProfileHistory} ProfileHistory
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileHistory.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ProfileHistory();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.name = reader.string();
                            break;
                        }
                    case 2: {
                            message.activeGeneration = reader.uint32();
                            break;
                        }
                    case 3: {
                            if (!(message.generations && message.generations.length))
                                message.generations = [];
                            message.generations.push($root.revault.bindings.ProfileGeneration.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ProfileHistory message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ProfileHistory} ProfileHistory
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileHistory.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ProfileHistory message.
             * @function verify
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ProfileHistory.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.activeGeneration != null && Object.hasOwnProperty.call(message, "activeGeneration"))
                    if (!$util.isInteger(message.activeGeneration))
                        return "activeGeneration: integer expected";
                if (message.generations != null && Object.hasOwnProperty.call(message, "generations")) {
                    if (!Array.isArray(message.generations))
                        return "generations: array expected";
                    for (let i = 0; i < message.generations.length; ++i) {
                        let error = $root.revault.bindings.ProfileGeneration.verify(message.generations[i], long + 1);
                        if (error)
                            return "generations." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a ProfileHistory message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ProfileHistory} ProfileHistory
             */
            ProfileHistory.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ProfileHistory)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ProfileHistory: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ProfileHistory();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.activeGeneration != null)
                    message.activeGeneration = object.activeGeneration >>> 0;
                if (object.generations) {
                    if (!Array.isArray(object.generations))
                        throw TypeError(".revault.bindings.ProfileHistory.generations: array expected");
                    message.generations = [];
                    for (let i = 0; i < object.generations.length; ++i) {
                        if (!$util.isObject(object.generations[i]))
                            throw TypeError(".revault.bindings.ProfileHistory.generations: object expected");
                        message.generations[i] = $root.revault.bindings.ProfileGeneration.fromObject(object.generations[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a ProfileHistory message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {revault.bindings.ProfileHistory} message ProfileHistory
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ProfileHistory.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.generations = [];
                if (options.defaults) {
                    object.name = "";
                    object.activeGeneration = 0;
                }
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.activeGeneration != null && Object.hasOwnProperty.call(message, "activeGeneration"))
                    object.activeGeneration = message.activeGeneration;
                if (message.generations && message.generations.length) {
                    object.generations = [];
                    for (let j = 0; j < message.generations.length; ++j)
                        object.generations[j] = $root.revault.bindings.ProfileGeneration.toObject(message.generations[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this ProfileHistory to JSON.
             * @function toJSON
             * @memberof revault.bindings.ProfileHistory
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ProfileHistory.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ProfileHistory
             * @function getTypeUrl
             * @memberof revault.bindings.ProfileHistory
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ProfileHistory.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ProfileHistory";
            };

            return ProfileHistory;
        })();

        bindings.KnownLockbox = (function() {

            /**
             * Properties of a KnownLockbox.
             * @memberof revault.bindings
             * @interface IKnownLockbox
             * @property {Uint8Array|null} [lockboxId] KnownLockbox lockboxId
             * @property {string|null} [path] KnownLockbox path
             * @property {number|Long|null} [lastSeenUnixMs] KnownLockbox lastSeenUnixMs
             */

            /**
             * Constructs a new KnownLockbox.
             * @memberof revault.bindings
             * @classdesc Represents a KnownLockbox.
             * @implements IKnownLockbox
             * @constructor
             * @param {revault.bindings.IKnownLockbox=} [properties] Properties to set
             */
            function KnownLockbox(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * KnownLockbox lockboxId.
             * @member {Uint8Array} lockboxId
             * @memberof revault.bindings.KnownLockbox
             * @instance
             */
            KnownLockbox.prototype.lockboxId = $util.newBuffer([]);

            /**
             * KnownLockbox path.
             * @member {string} path
             * @memberof revault.bindings.KnownLockbox
             * @instance
             */
            KnownLockbox.prototype.path = "";

            /**
             * KnownLockbox lastSeenUnixMs.
             * @member {number|Long} lastSeenUnixMs
             * @memberof revault.bindings.KnownLockbox
             * @instance
             */
            KnownLockbox.prototype.lastSeenUnixMs = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new KnownLockbox instance using the specified properties.
             * @function create
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {revault.bindings.IKnownLockbox=} [properties] Properties to set
             * @returns {revault.bindings.KnownLockbox} KnownLockbox instance
             */
            KnownLockbox.create = function create(properties) {
                return new KnownLockbox(properties);
            };

            /**
             * Encodes the specified KnownLockbox message. Does not implicitly {@link revault.bindings.KnownLockbox.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {revault.bindings.IKnownLockbox} message KnownLockbox message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KnownLockbox.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.lockboxId);
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.path);
                if (message.lastSeenUnixMs != null && Object.hasOwnProperty.call(message, "lastSeenUnixMs"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.lastSeenUnixMs);
                return writer;
            };

            /**
             * Encodes the specified KnownLockbox message, length delimited. Does not implicitly {@link revault.bindings.KnownLockbox.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {revault.bindings.IKnownLockbox} message KnownLockbox message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KnownLockbox.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a KnownLockbox message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.KnownLockbox} KnownLockbox
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KnownLockbox.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.KnownLockbox();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.lockboxId = reader.bytes();
                            break;
                        }
                    case 2: {
                            message.path = reader.string();
                            break;
                        }
                    case 3: {
                            message.lastSeenUnixMs = reader.uint64();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a KnownLockbox message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.KnownLockbox} KnownLockbox
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KnownLockbox.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a KnownLockbox message.
             * @function verify
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            KnownLockbox.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    if (!(message.lockboxId && typeof message.lockboxId.length === "number" || $util.isString(message.lockboxId)))
                        return "lockboxId: buffer expected";
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    if (!$util.isString(message.path))
                        return "path: string expected";
                if (message.lastSeenUnixMs != null && Object.hasOwnProperty.call(message, "lastSeenUnixMs"))
                    if (!$util.isInteger(message.lastSeenUnixMs) && !(message.lastSeenUnixMs && $util.isInteger(message.lastSeenUnixMs.low) && $util.isInteger(message.lastSeenUnixMs.high)))
                        return "lastSeenUnixMs: integer|Long expected";
                return null;
            };

            /**
             * Creates a KnownLockbox message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.KnownLockbox} KnownLockbox
             */
            KnownLockbox.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.KnownLockbox)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.KnownLockbox: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.KnownLockbox();
                if (object.lockboxId != null)
                    if (typeof object.lockboxId === "string")
                        $util.base64.decode(object.lockboxId, message.lockboxId = $util.newBuffer($util.base64.length(object.lockboxId)), 0);
                    else if (object.lockboxId.length >= 0)
                        message.lockboxId = object.lockboxId;
                if (object.path != null)
                    message.path = String(object.path);
                if (object.lastSeenUnixMs != null)
                    if ($util.Long)
                        message.lastSeenUnixMs = $util.Long.fromValue(object.lastSeenUnixMs, true);
                    else if (typeof object.lastSeenUnixMs === "string")
                        message.lastSeenUnixMs = parseInt(object.lastSeenUnixMs, 10);
                    else if (typeof object.lastSeenUnixMs === "number")
                        message.lastSeenUnixMs = object.lastSeenUnixMs;
                    else if (typeof object.lastSeenUnixMs === "object")
                        message.lastSeenUnixMs = new $util.LongBits(object.lastSeenUnixMs.low >>> 0, object.lastSeenUnixMs.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from a KnownLockbox message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {revault.bindings.KnownLockbox} message KnownLockbox
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            KnownLockbox.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.lockboxId = "";
                    else {
                        object.lockboxId = [];
                        if (options.bytes !== Array)
                            object.lockboxId = $util.newBuffer(object.lockboxId);
                    }
                    object.path = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.lastSeenUnixMs = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.lastSeenUnixMs = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    object.lockboxId = options.bytes === String ? $util.base64.encode(message.lockboxId, 0, message.lockboxId.length) : options.bytes === Array ? Array.prototype.slice.call(message.lockboxId) : message.lockboxId;
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    object.path = message.path;
                if (message.lastSeenUnixMs != null && Object.hasOwnProperty.call(message, "lastSeenUnixMs"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.lastSeenUnixMs = typeof message.lastSeenUnixMs === "number" ? BigInt(message.lastSeenUnixMs) : $util.Long.fromBits(message.lastSeenUnixMs.low >>> 0, message.lastSeenUnixMs.high >>> 0, true).toBigInt();
                    else if (typeof message.lastSeenUnixMs === "number")
                        object.lastSeenUnixMs = options.longs === String ? String(message.lastSeenUnixMs) : message.lastSeenUnixMs;
                    else
                        object.lastSeenUnixMs = options.longs === String ? $util.Long.prototype.toString.call(message.lastSeenUnixMs) : options.longs === Number ? new $util.LongBits(message.lastSeenUnixMs.low >>> 0, message.lastSeenUnixMs.high >>> 0).toNumber(true) : message.lastSeenUnixMs;
                return object;
            };

            /**
             * Converts this KnownLockbox to JSON.
             * @function toJSON
             * @memberof revault.bindings.KnownLockbox
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            KnownLockbox.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for KnownLockbox
             * @function getTypeUrl
             * @memberof revault.bindings.KnownLockbox
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            KnownLockbox.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.KnownLockbox";
            };

            return KnownLockbox;
        })();

        bindings.KnownLockboxList = (function() {

            /**
             * Properties of a KnownLockboxList.
             * @memberof revault.bindings
             * @interface IKnownLockboxList
             * @property {Array.<revault.bindings.IKnownLockbox>|null} [values] KnownLockboxList values
             */

            /**
             * Constructs a new KnownLockboxList.
             * @memberof revault.bindings
             * @classdesc Represents a KnownLockboxList.
             * @implements IKnownLockboxList
             * @constructor
             * @param {revault.bindings.IKnownLockboxList=} [properties] Properties to set
             */
            function KnownLockboxList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * KnownLockboxList values.
             * @member {Array.<revault.bindings.IKnownLockbox>} values
             * @memberof revault.bindings.KnownLockboxList
             * @instance
             */
            KnownLockboxList.prototype.values = $util.emptyArray;

            /**
             * Creates a new KnownLockboxList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {revault.bindings.IKnownLockboxList=} [properties] Properties to set
             * @returns {revault.bindings.KnownLockboxList} KnownLockboxList instance
             */
            KnownLockboxList.create = function create(properties) {
                return new KnownLockboxList(properties);
            };

            /**
             * Encodes the specified KnownLockboxList message. Does not implicitly {@link revault.bindings.KnownLockboxList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {revault.bindings.IKnownLockboxList} message KnownLockboxList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KnownLockboxList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.KnownLockbox.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified KnownLockboxList message, length delimited. Does not implicitly {@link revault.bindings.KnownLockboxList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {revault.bindings.IKnownLockboxList} message KnownLockboxList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KnownLockboxList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a KnownLockboxList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.KnownLockboxList} KnownLockboxList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KnownLockboxList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.KnownLockboxList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.KnownLockbox.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a KnownLockboxList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.KnownLockboxList} KnownLockboxList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KnownLockboxList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a KnownLockboxList message.
             * @function verify
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            KnownLockboxList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.KnownLockbox.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a KnownLockboxList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.KnownLockboxList} KnownLockboxList
             */
            KnownLockboxList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.KnownLockboxList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.KnownLockboxList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.KnownLockboxList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.KnownLockboxList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.KnownLockboxList.values: object expected");
                        message.values[i] = $root.revault.bindings.KnownLockbox.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a KnownLockboxList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {revault.bindings.KnownLockboxList} message KnownLockboxList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            KnownLockboxList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.KnownLockbox.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this KnownLockboxList to JSON.
             * @function toJSON
             * @memberof revault.bindings.KnownLockboxList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            KnownLockboxList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for KnownLockboxList
             * @function getTypeUrl
             * @memberof revault.bindings.KnownLockboxList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            KnownLockboxList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.KnownLockboxList";
            };

            return KnownLockboxList;
        })();

        bindings.AccessSlotLabel = (function() {

            /**
             * Properties of an AccessSlotLabel.
             * @memberof revault.bindings
             * @interface IAccessSlotLabel
             * @property {Uint8Array|null} [lockboxId] AccessSlotLabel lockboxId
             * @property {number|Long|null} [slotId] AccessSlotLabel slotId
             * @property {string|null} [name] AccessSlotLabel name
             * @property {number|Long|null} [updatedAtUnixMs] AccessSlotLabel updatedAtUnixMs
             */

            /**
             * Constructs a new AccessSlotLabel.
             * @memberof revault.bindings
             * @classdesc Represents an AccessSlotLabel.
             * @implements IAccessSlotLabel
             * @constructor
             * @param {revault.bindings.IAccessSlotLabel=} [properties] Properties to set
             */
            function AccessSlotLabel(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AccessSlotLabel lockboxId.
             * @member {Uint8Array} lockboxId
             * @memberof revault.bindings.AccessSlotLabel
             * @instance
             */
            AccessSlotLabel.prototype.lockboxId = $util.newBuffer([]);

            /**
             * AccessSlotLabel slotId.
             * @member {number|Long} slotId
             * @memberof revault.bindings.AccessSlotLabel
             * @instance
             */
            AccessSlotLabel.prototype.slotId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * AccessSlotLabel name.
             * @member {string} name
             * @memberof revault.bindings.AccessSlotLabel
             * @instance
             */
            AccessSlotLabel.prototype.name = "";

            /**
             * AccessSlotLabel updatedAtUnixMs.
             * @member {number|Long} updatedAtUnixMs
             * @memberof revault.bindings.AccessSlotLabel
             * @instance
             */
            AccessSlotLabel.prototype.updatedAtUnixMs = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new AccessSlotLabel instance using the specified properties.
             * @function create
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {revault.bindings.IAccessSlotLabel=} [properties] Properties to set
             * @returns {revault.bindings.AccessSlotLabel} AccessSlotLabel instance
             */
            AccessSlotLabel.create = function create(properties) {
                return new AccessSlotLabel(properties);
            };

            /**
             * Encodes the specified AccessSlotLabel message. Does not implicitly {@link revault.bindings.AccessSlotLabel.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {revault.bindings.IAccessSlotLabel} message AccessSlotLabel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AccessSlotLabel.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.lockboxId);
                if (message.slotId != null && Object.hasOwnProperty.call(message, "slotId"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.slotId);
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.name);
                if (message.updatedAtUnixMs != null && Object.hasOwnProperty.call(message, "updatedAtUnixMs"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.updatedAtUnixMs);
                return writer;
            };

            /**
             * Encodes the specified AccessSlotLabel message, length delimited. Does not implicitly {@link revault.bindings.AccessSlotLabel.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {revault.bindings.IAccessSlotLabel} message AccessSlotLabel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AccessSlotLabel.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AccessSlotLabel message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.AccessSlotLabel} AccessSlotLabel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AccessSlotLabel.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.AccessSlotLabel();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.lockboxId = reader.bytes();
                            break;
                        }
                    case 2: {
                            message.slotId = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.name = reader.string();
                            break;
                        }
                    case 4: {
                            message.updatedAtUnixMs = reader.uint64();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an AccessSlotLabel message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.AccessSlotLabel} AccessSlotLabel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AccessSlotLabel.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AccessSlotLabel message.
             * @function verify
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AccessSlotLabel.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    if (!(message.lockboxId && typeof message.lockboxId.length === "number" || $util.isString(message.lockboxId)))
                        return "lockboxId: buffer expected";
                if (message.slotId != null && Object.hasOwnProperty.call(message, "slotId"))
                    if (!$util.isInteger(message.slotId) && !(message.slotId && $util.isInteger(message.slotId.low) && $util.isInteger(message.slotId.high)))
                        return "slotId: integer|Long expected";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.updatedAtUnixMs != null && Object.hasOwnProperty.call(message, "updatedAtUnixMs"))
                    if (!$util.isInteger(message.updatedAtUnixMs) && !(message.updatedAtUnixMs && $util.isInteger(message.updatedAtUnixMs.low) && $util.isInteger(message.updatedAtUnixMs.high)))
                        return "updatedAtUnixMs: integer|Long expected";
                return null;
            };

            /**
             * Creates an AccessSlotLabel message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.AccessSlotLabel} AccessSlotLabel
             */
            AccessSlotLabel.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.AccessSlotLabel)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.AccessSlotLabel: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.AccessSlotLabel();
                if (object.lockboxId != null)
                    if (typeof object.lockboxId === "string")
                        $util.base64.decode(object.lockboxId, message.lockboxId = $util.newBuffer($util.base64.length(object.lockboxId)), 0);
                    else if (object.lockboxId.length >= 0)
                        message.lockboxId = object.lockboxId;
                if (object.slotId != null)
                    if ($util.Long)
                        message.slotId = $util.Long.fromValue(object.slotId, true);
                    else if (typeof object.slotId === "string")
                        message.slotId = parseInt(object.slotId, 10);
                    else if (typeof object.slotId === "number")
                        message.slotId = object.slotId;
                    else if (typeof object.slotId === "object")
                        message.slotId = new $util.LongBits(object.slotId.low >>> 0, object.slotId.high >>> 0).toNumber(true);
                if (object.name != null)
                    message.name = String(object.name);
                if (object.updatedAtUnixMs != null)
                    if ($util.Long)
                        message.updatedAtUnixMs = $util.Long.fromValue(object.updatedAtUnixMs, true);
                    else if (typeof object.updatedAtUnixMs === "string")
                        message.updatedAtUnixMs = parseInt(object.updatedAtUnixMs, 10);
                    else if (typeof object.updatedAtUnixMs === "number")
                        message.updatedAtUnixMs = object.updatedAtUnixMs;
                    else if (typeof object.updatedAtUnixMs === "object")
                        message.updatedAtUnixMs = new $util.LongBits(object.updatedAtUnixMs.low >>> 0, object.updatedAtUnixMs.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from an AccessSlotLabel message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {revault.bindings.AccessSlotLabel} message AccessSlotLabel
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AccessSlotLabel.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.lockboxId = "";
                    else {
                        object.lockboxId = [];
                        if (options.bytes !== Array)
                            object.lockboxId = $util.newBuffer(object.lockboxId);
                    }
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.slotId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.slotId = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.name = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.updatedAtUnixMs = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.updatedAtUnixMs = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.lockboxId != null && Object.hasOwnProperty.call(message, "lockboxId"))
                    object.lockboxId = options.bytes === String ? $util.base64.encode(message.lockboxId, 0, message.lockboxId.length) : options.bytes === Array ? Array.prototype.slice.call(message.lockboxId) : message.lockboxId;
                if (message.slotId != null && Object.hasOwnProperty.call(message, "slotId"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.slotId = typeof message.slotId === "number" ? BigInt(message.slotId) : $util.Long.fromBits(message.slotId.low >>> 0, message.slotId.high >>> 0, true).toBigInt();
                    else if (typeof message.slotId === "number")
                        object.slotId = options.longs === String ? String(message.slotId) : message.slotId;
                    else
                        object.slotId = options.longs === String ? $util.Long.prototype.toString.call(message.slotId) : options.longs === Number ? new $util.LongBits(message.slotId.low >>> 0, message.slotId.high >>> 0).toNumber(true) : message.slotId;
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.updatedAtUnixMs != null && Object.hasOwnProperty.call(message, "updatedAtUnixMs"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.updatedAtUnixMs = typeof message.updatedAtUnixMs === "number" ? BigInt(message.updatedAtUnixMs) : $util.Long.fromBits(message.updatedAtUnixMs.low >>> 0, message.updatedAtUnixMs.high >>> 0, true).toBigInt();
                    else if (typeof message.updatedAtUnixMs === "number")
                        object.updatedAtUnixMs = options.longs === String ? String(message.updatedAtUnixMs) : message.updatedAtUnixMs;
                    else
                        object.updatedAtUnixMs = options.longs === String ? $util.Long.prototype.toString.call(message.updatedAtUnixMs) : options.longs === Number ? new $util.LongBits(message.updatedAtUnixMs.low >>> 0, message.updatedAtUnixMs.high >>> 0).toNumber(true) : message.updatedAtUnixMs;
                return object;
            };

            /**
             * Converts this AccessSlotLabel to JSON.
             * @function toJSON
             * @memberof revault.bindings.AccessSlotLabel
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AccessSlotLabel.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for AccessSlotLabel
             * @function getTypeUrl
             * @memberof revault.bindings.AccessSlotLabel
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            AccessSlotLabel.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.AccessSlotLabel";
            };

            return AccessSlotLabel;
        })();

        bindings.AccessSlotLabelList = (function() {

            /**
             * Properties of an AccessSlotLabelList.
             * @memberof revault.bindings
             * @interface IAccessSlotLabelList
             * @property {Array.<revault.bindings.IAccessSlotLabel>|null} [values] AccessSlotLabelList values
             */

            /**
             * Constructs a new AccessSlotLabelList.
             * @memberof revault.bindings
             * @classdesc Represents an AccessSlotLabelList.
             * @implements IAccessSlotLabelList
             * @constructor
             * @param {revault.bindings.IAccessSlotLabelList=} [properties] Properties to set
             */
            function AccessSlotLabelList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AccessSlotLabelList values.
             * @member {Array.<revault.bindings.IAccessSlotLabel>} values
             * @memberof revault.bindings.AccessSlotLabelList
             * @instance
             */
            AccessSlotLabelList.prototype.values = $util.emptyArray;

            /**
             * Creates a new AccessSlotLabelList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {revault.bindings.IAccessSlotLabelList=} [properties] Properties to set
             * @returns {revault.bindings.AccessSlotLabelList} AccessSlotLabelList instance
             */
            AccessSlotLabelList.create = function create(properties) {
                return new AccessSlotLabelList(properties);
            };

            /**
             * Encodes the specified AccessSlotLabelList message. Does not implicitly {@link revault.bindings.AccessSlotLabelList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {revault.bindings.IAccessSlotLabelList} message AccessSlotLabelList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AccessSlotLabelList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.AccessSlotLabel.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified AccessSlotLabelList message, length delimited. Does not implicitly {@link revault.bindings.AccessSlotLabelList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {revault.bindings.IAccessSlotLabelList} message AccessSlotLabelList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AccessSlotLabelList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AccessSlotLabelList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.AccessSlotLabelList} AccessSlotLabelList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AccessSlotLabelList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.AccessSlotLabelList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.AccessSlotLabel.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an AccessSlotLabelList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.AccessSlotLabelList} AccessSlotLabelList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AccessSlotLabelList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AccessSlotLabelList message.
             * @function verify
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AccessSlotLabelList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.AccessSlotLabel.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an AccessSlotLabelList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.AccessSlotLabelList} AccessSlotLabelList
             */
            AccessSlotLabelList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.AccessSlotLabelList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.AccessSlotLabelList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.AccessSlotLabelList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.AccessSlotLabelList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.AccessSlotLabelList.values: object expected");
                        message.values[i] = $root.revault.bindings.AccessSlotLabel.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an AccessSlotLabelList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {revault.bindings.AccessSlotLabelList} message AccessSlotLabelList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AccessSlotLabelList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.AccessSlotLabel.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this AccessSlotLabelList to JSON.
             * @function toJSON
             * @memberof revault.bindings.AccessSlotLabelList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AccessSlotLabelList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for AccessSlotLabelList
             * @function getTypeUrl
             * @memberof revault.bindings.AccessSlotLabelList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            AccessSlotLabelList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.AccessSlotLabelList";
            };

            return AccessSlotLabelList;
        })();

        bindings.StreamChunk = (function() {

            /**
             * Properties of a StreamChunk.
             * @memberof revault.bindings
             * @interface IStreamChunk
             * @property {string|null} [path] StreamChunk path
             * @property {number|Long|null} [fileOffset] StreamChunk fileOffset
             * @property {number|Long|null} [length] StreamChunk length
             * @property {number|Long|null} [physicalOffset] StreamChunk physicalOffset
             * @property {boolean|null} [sparse] StreamChunk sparse
             * @property {Uint8Array|null} [data] StreamChunk data
             */

            /**
             * Constructs a new StreamChunk.
             * @memberof revault.bindings
             * @classdesc Represents a StreamChunk.
             * @implements IStreamChunk
             * @constructor
             * @param {revault.bindings.IStreamChunk=} [properties] Properties to set
             */
            function StreamChunk(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * StreamChunk path.
             * @member {string} path
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.path = "";

            /**
             * StreamChunk fileOffset.
             * @member {number|Long} fileOffset
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.fileOffset = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * StreamChunk length.
             * @member {number|Long} length
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.length = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * StreamChunk physicalOffset.
             * @member {number|Long} physicalOffset
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.physicalOffset = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * StreamChunk sparse.
             * @member {boolean} sparse
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.sparse = false;

            /**
             * StreamChunk data.
             * @member {Uint8Array} data
             * @memberof revault.bindings.StreamChunk
             * @instance
             */
            StreamChunk.prototype.data = $util.newBuffer([]);

            /**
             * Creates a new StreamChunk instance using the specified properties.
             * @function create
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {revault.bindings.IStreamChunk=} [properties] Properties to set
             * @returns {revault.bindings.StreamChunk} StreamChunk instance
             */
            StreamChunk.create = function create(properties) {
                return new StreamChunk(properties);
            };

            /**
             * Encodes the specified StreamChunk message. Does not implicitly {@link revault.bindings.StreamChunk.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {revault.bindings.IStreamChunk} message StreamChunk message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamChunk.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.path);
                if (message.fileOffset != null && Object.hasOwnProperty.call(message, "fileOffset"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.fileOffset);
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.length);
                if (message.physicalOffset != null && Object.hasOwnProperty.call(message, "physicalOffset"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.physicalOffset);
                if (message.sparse != null && Object.hasOwnProperty.call(message, "sparse"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.sparse);
                if (message.data != null && Object.hasOwnProperty.call(message, "data"))
                    writer.uint32(/* id 6, wireType 2 =*/50).bytes(message.data);
                return writer;
            };

            /**
             * Encodes the specified StreamChunk message, length delimited. Does not implicitly {@link revault.bindings.StreamChunk.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {revault.bindings.IStreamChunk} message StreamChunk message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamChunk.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a StreamChunk message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.StreamChunk} StreamChunk
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamChunk.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.StreamChunk();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.path = reader.string();
                            break;
                        }
                    case 2: {
                            message.fileOffset = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.length = reader.uint64();
                            break;
                        }
                    case 4: {
                            message.physicalOffset = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.sparse = reader.bool();
                            break;
                        }
                    case 6: {
                            message.data = reader.bytes();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a StreamChunk message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.StreamChunk} StreamChunk
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamChunk.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a StreamChunk message.
             * @function verify
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StreamChunk.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    if (!$util.isString(message.path))
                        return "path: string expected";
                if (message.fileOffset != null && Object.hasOwnProperty.call(message, "fileOffset"))
                    if (!$util.isInteger(message.fileOffset) && !(message.fileOffset && $util.isInteger(message.fileOffset.low) && $util.isInteger(message.fileOffset.high)))
                        return "fileOffset: integer|Long expected";
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    if (!$util.isInteger(message.length) && !(message.length && $util.isInteger(message.length.low) && $util.isInteger(message.length.high)))
                        return "length: integer|Long expected";
                if (message.physicalOffset != null && Object.hasOwnProperty.call(message, "physicalOffset"))
                    if (!$util.isInteger(message.physicalOffset) && !(message.physicalOffset && $util.isInteger(message.physicalOffset.low) && $util.isInteger(message.physicalOffset.high)))
                        return "physicalOffset: integer|Long expected";
                if (message.sparse != null && Object.hasOwnProperty.call(message, "sparse"))
                    if (typeof message.sparse !== "boolean")
                        return "sparse: boolean expected";
                if (message.data != null && Object.hasOwnProperty.call(message, "data"))
                    if (!(message.data && typeof message.data.length === "number" || $util.isString(message.data)))
                        return "data: buffer expected";
                return null;
            };

            /**
             * Creates a StreamChunk message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.StreamChunk} StreamChunk
             */
            StreamChunk.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.StreamChunk)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.StreamChunk: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.StreamChunk();
                if (object.path != null)
                    message.path = String(object.path);
                if (object.fileOffset != null)
                    if ($util.Long)
                        message.fileOffset = $util.Long.fromValue(object.fileOffset, true);
                    else if (typeof object.fileOffset === "string")
                        message.fileOffset = parseInt(object.fileOffset, 10);
                    else if (typeof object.fileOffset === "number")
                        message.fileOffset = object.fileOffset;
                    else if (typeof object.fileOffset === "object")
                        message.fileOffset = new $util.LongBits(object.fileOffset.low >>> 0, object.fileOffset.high >>> 0).toNumber(true);
                if (object.length != null)
                    if ($util.Long)
                        message.length = $util.Long.fromValue(object.length, true);
                    else if (typeof object.length === "string")
                        message.length = parseInt(object.length, 10);
                    else if (typeof object.length === "number")
                        message.length = object.length;
                    else if (typeof object.length === "object")
                        message.length = new $util.LongBits(object.length.low >>> 0, object.length.high >>> 0).toNumber(true);
                if (object.physicalOffset != null)
                    if ($util.Long)
                        message.physicalOffset = $util.Long.fromValue(object.physicalOffset, true);
                    else if (typeof object.physicalOffset === "string")
                        message.physicalOffset = parseInt(object.physicalOffset, 10);
                    else if (typeof object.physicalOffset === "number")
                        message.physicalOffset = object.physicalOffset;
                    else if (typeof object.physicalOffset === "object")
                        message.physicalOffset = new $util.LongBits(object.physicalOffset.low >>> 0, object.physicalOffset.high >>> 0).toNumber(true);
                if (object.sparse != null)
                    message.sparse = Boolean(object.sparse);
                if (object.data != null)
                    if (typeof object.data === "string")
                        $util.base64.decode(object.data, message.data = $util.newBuffer($util.base64.length(object.data)), 0);
                    else if (object.data.length >= 0)
                        message.data = object.data;
                return message;
            };

            /**
             * Creates a plain object from a StreamChunk message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {revault.bindings.StreamChunk} message StreamChunk
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StreamChunk.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.path = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.fileOffset = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.fileOffset = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.length = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.length = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.physicalOffset = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.physicalOffset = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.sparse = false;
                    if (options.bytes === String)
                        object.data = "";
                    else {
                        object.data = [];
                        if (options.bytes !== Array)
                            object.data = $util.newBuffer(object.data);
                    }
                }
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    object.path = message.path;
                if (message.fileOffset != null && Object.hasOwnProperty.call(message, "fileOffset"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.fileOffset = typeof message.fileOffset === "number" ? BigInt(message.fileOffset) : $util.Long.fromBits(message.fileOffset.low >>> 0, message.fileOffset.high >>> 0, true).toBigInt();
                    else if (typeof message.fileOffset === "number")
                        object.fileOffset = options.longs === String ? String(message.fileOffset) : message.fileOffset;
                    else
                        object.fileOffset = options.longs === String ? $util.Long.prototype.toString.call(message.fileOffset) : options.longs === Number ? new $util.LongBits(message.fileOffset.low >>> 0, message.fileOffset.high >>> 0).toNumber(true) : message.fileOffset;
                if (message.length != null && Object.hasOwnProperty.call(message, "length"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.length = typeof message.length === "number" ? BigInt(message.length) : $util.Long.fromBits(message.length.low >>> 0, message.length.high >>> 0, true).toBigInt();
                    else if (typeof message.length === "number")
                        object.length = options.longs === String ? String(message.length) : message.length;
                    else
                        object.length = options.longs === String ? $util.Long.prototype.toString.call(message.length) : options.longs === Number ? new $util.LongBits(message.length.low >>> 0, message.length.high >>> 0).toNumber(true) : message.length;
                if (message.physicalOffset != null && Object.hasOwnProperty.call(message, "physicalOffset"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.physicalOffset = typeof message.physicalOffset === "number" ? BigInt(message.physicalOffset) : $util.Long.fromBits(message.physicalOffset.low >>> 0, message.physicalOffset.high >>> 0, true).toBigInt();
                    else if (typeof message.physicalOffset === "number")
                        object.physicalOffset = options.longs === String ? String(message.physicalOffset) : message.physicalOffset;
                    else
                        object.physicalOffset = options.longs === String ? $util.Long.prototype.toString.call(message.physicalOffset) : options.longs === Number ? new $util.LongBits(message.physicalOffset.low >>> 0, message.physicalOffset.high >>> 0).toNumber(true) : message.physicalOffset;
                if (message.sparse != null && Object.hasOwnProperty.call(message, "sparse"))
                    object.sparse = message.sparse;
                if (message.data != null && Object.hasOwnProperty.call(message, "data"))
                    object.data = options.bytes === String ? $util.base64.encode(message.data, 0, message.data.length) : options.bytes === Array ? Array.prototype.slice.call(message.data) : message.data;
                return object;
            };

            /**
             * Converts this StreamChunk to JSON.
             * @function toJSON
             * @memberof revault.bindings.StreamChunk
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StreamChunk.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for StreamChunk
             * @function getTypeUrl
             * @memberof revault.bindings.StreamChunk
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            StreamChunk.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.StreamChunk";
            };

            return StreamChunk;
        })();

        bindings.StreamChunkList = (function() {

            /**
             * Properties of a StreamChunkList.
             * @memberof revault.bindings
             * @interface IStreamChunkList
             * @property {Array.<revault.bindings.IStreamChunk>|null} [values] StreamChunkList values
             */

            /**
             * Constructs a new StreamChunkList.
             * @memberof revault.bindings
             * @classdesc Represents a StreamChunkList.
             * @implements IStreamChunkList
             * @constructor
             * @param {revault.bindings.IStreamChunkList=} [properties] Properties to set
             */
            function StreamChunkList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * StreamChunkList values.
             * @member {Array.<revault.bindings.IStreamChunk>} values
             * @memberof revault.bindings.StreamChunkList
             * @instance
             */
            StreamChunkList.prototype.values = $util.emptyArray;

            /**
             * Creates a new StreamChunkList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {revault.bindings.IStreamChunkList=} [properties] Properties to set
             * @returns {revault.bindings.StreamChunkList} StreamChunkList instance
             */
            StreamChunkList.create = function create(properties) {
                return new StreamChunkList(properties);
            };

            /**
             * Encodes the specified StreamChunkList message. Does not implicitly {@link revault.bindings.StreamChunkList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {revault.bindings.IStreamChunkList} message StreamChunkList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamChunkList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.StreamChunk.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified StreamChunkList message, length delimited. Does not implicitly {@link revault.bindings.StreamChunkList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {revault.bindings.IStreamChunkList} message StreamChunkList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamChunkList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a StreamChunkList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.StreamChunkList} StreamChunkList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamChunkList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.StreamChunkList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.StreamChunk.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a StreamChunkList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.StreamChunkList} StreamChunkList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamChunkList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a StreamChunkList message.
             * @function verify
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StreamChunkList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.StreamChunk.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a StreamChunkList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.StreamChunkList} StreamChunkList
             */
            StreamChunkList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.StreamChunkList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.StreamChunkList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.StreamChunkList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.StreamChunkList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.StreamChunkList.values: object expected");
                        message.values[i] = $root.revault.bindings.StreamChunk.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a StreamChunkList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {revault.bindings.StreamChunkList} message StreamChunkList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StreamChunkList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.StreamChunk.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this StreamChunkList to JSON.
             * @function toJSON
             * @memberof revault.bindings.StreamChunkList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StreamChunkList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for StreamChunkList
             * @function getTypeUrl
             * @memberof revault.bindings.StreamChunkList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            StreamChunkList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.StreamChunkList";
            };

            return StreamChunkList;
        })();

        bindings.RuntimeOptions = (function() {

            /**
             * Properties of a RuntimeOptions.
             * @memberof revault.bindings
             * @interface IRuntimeOptions
             * @property {string|null} [workloadProfile] RuntimeOptions workloadProfile
             * @property {string|null} [workerPolicy] RuntimeOptions workerPolicy
             */

            /**
             * Constructs a new RuntimeOptions.
             * @memberof revault.bindings
             * @classdesc Represents a RuntimeOptions.
             * @implements IRuntimeOptions
             * @constructor
             * @param {revault.bindings.IRuntimeOptions=} [properties] Properties to set
             */
            function RuntimeOptions(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * RuntimeOptions workloadProfile.
             * @member {string} workloadProfile
             * @memberof revault.bindings.RuntimeOptions
             * @instance
             */
            RuntimeOptions.prototype.workloadProfile = "";

            /**
             * RuntimeOptions workerPolicy.
             * @member {string} workerPolicy
             * @memberof revault.bindings.RuntimeOptions
             * @instance
             */
            RuntimeOptions.prototype.workerPolicy = "";

            /**
             * Creates a new RuntimeOptions instance using the specified properties.
             * @function create
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {revault.bindings.IRuntimeOptions=} [properties] Properties to set
             * @returns {revault.bindings.RuntimeOptions} RuntimeOptions instance
             */
            RuntimeOptions.create = function create(properties) {
                return new RuntimeOptions(properties);
            };

            /**
             * Encodes the specified RuntimeOptions message. Does not implicitly {@link revault.bindings.RuntimeOptions.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {revault.bindings.IRuntimeOptions} message RuntimeOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RuntimeOptions.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.workloadProfile != null && Object.hasOwnProperty.call(message, "workloadProfile"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.workloadProfile);
                if (message.workerPolicy != null && Object.hasOwnProperty.call(message, "workerPolicy"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.workerPolicy);
                return writer;
            };

            /**
             * Encodes the specified RuntimeOptions message, length delimited. Does not implicitly {@link revault.bindings.RuntimeOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {revault.bindings.IRuntimeOptions} message RuntimeOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RuntimeOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a RuntimeOptions message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.RuntimeOptions} RuntimeOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RuntimeOptions.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.RuntimeOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.workloadProfile = reader.string();
                            break;
                        }
                    case 2: {
                            message.workerPolicy = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a RuntimeOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.RuntimeOptions} RuntimeOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RuntimeOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a RuntimeOptions message.
             * @function verify
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RuntimeOptions.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.workloadProfile != null && Object.hasOwnProperty.call(message, "workloadProfile"))
                    if (!$util.isString(message.workloadProfile))
                        return "workloadProfile: string expected";
                if (message.workerPolicy != null && Object.hasOwnProperty.call(message, "workerPolicy"))
                    if (!$util.isString(message.workerPolicy))
                        return "workerPolicy: string expected";
                return null;
            };

            /**
             * Creates a RuntimeOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.RuntimeOptions} RuntimeOptions
             */
            RuntimeOptions.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.RuntimeOptions)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.RuntimeOptions: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.RuntimeOptions();
                if (object.workloadProfile != null)
                    message.workloadProfile = String(object.workloadProfile);
                if (object.workerPolicy != null)
                    message.workerPolicy = String(object.workerPolicy);
                return message;
            };

            /**
             * Creates a plain object from a RuntimeOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {revault.bindings.RuntimeOptions} message RuntimeOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RuntimeOptions.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.workloadProfile = "";
                    object.workerPolicy = "";
                }
                if (message.workloadProfile != null && Object.hasOwnProperty.call(message, "workloadProfile"))
                    object.workloadProfile = message.workloadProfile;
                if (message.workerPolicy != null && Object.hasOwnProperty.call(message, "workerPolicy"))
                    object.workerPolicy = message.workerPolicy;
                return object;
            };

            /**
             * Converts this RuntimeOptions to JSON.
             * @function toJSON
             * @memberof revault.bindings.RuntimeOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RuntimeOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for RuntimeOptions
             * @function getTypeUrl
             * @memberof revault.bindings.RuntimeOptions
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            RuntimeOptions.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.RuntimeOptions";
            };

            return RuntimeOptions;
        })();

        bindings.Variable = (function() {

            /**
             * Properties of a Variable.
             * @memberof revault.bindings
             * @interface IVariable
             * @property {string|null} [name] Variable name
             * @property {string|null} [sensitivity] Variable sensitivity
             */

            /**
             * Constructs a new Variable.
             * @memberof revault.bindings
             * @classdesc Represents a Variable.
             * @implements IVariable
             * @constructor
             * @param {revault.bindings.IVariable=} [properties] Properties to set
             */
            function Variable(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Variable name.
             * @member {string} name
             * @memberof revault.bindings.Variable
             * @instance
             */
            Variable.prototype.name = "";

            /**
             * Variable sensitivity.
             * @member {string} sensitivity
             * @memberof revault.bindings.Variable
             * @instance
             */
            Variable.prototype.sensitivity = "";

            /**
             * Creates a new Variable instance using the specified properties.
             * @function create
             * @memberof revault.bindings.Variable
             * @static
             * @param {revault.bindings.IVariable=} [properties] Properties to set
             * @returns {revault.bindings.Variable} Variable instance
             */
            Variable.create = function create(properties) {
                return new Variable(properties);
            };

            /**
             * Encodes the specified Variable message. Does not implicitly {@link revault.bindings.Variable.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.Variable
             * @static
             * @param {revault.bindings.IVariable} message Variable message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Variable.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.sensitivity != null && Object.hasOwnProperty.call(message, "sensitivity"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.sensitivity);
                return writer;
            };

            /**
             * Encodes the specified Variable message, length delimited. Does not implicitly {@link revault.bindings.Variable.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.Variable
             * @static
             * @param {revault.bindings.IVariable} message Variable message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Variable.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Variable message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.Variable
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.Variable} Variable
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Variable.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.Variable();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.name = reader.string();
                            break;
                        }
                    case 2: {
                            message.sensitivity = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Variable message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.Variable
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.Variable} Variable
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Variable.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Variable message.
             * @function verify
             * @memberof revault.bindings.Variable
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Variable.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.sensitivity != null && Object.hasOwnProperty.call(message, "sensitivity"))
                    if (!$util.isString(message.sensitivity))
                        return "sensitivity: string expected";
                return null;
            };

            /**
             * Creates a Variable message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.Variable
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.Variable} Variable
             */
            Variable.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.Variable)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.Variable: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.Variable();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.sensitivity != null)
                    message.sensitivity = String(object.sensitivity);
                return message;
            };

            /**
             * Creates a plain object from a Variable message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.Variable
             * @static
             * @param {revault.bindings.Variable} message Variable
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Variable.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    object.sensitivity = "";
                }
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.sensitivity != null && Object.hasOwnProperty.call(message, "sensitivity"))
                    object.sensitivity = message.sensitivity;
                return object;
            };

            /**
             * Converts this Variable to JSON.
             * @function toJSON
             * @memberof revault.bindings.Variable
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Variable.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for Variable
             * @function getTypeUrl
             * @memberof revault.bindings.Variable
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            Variable.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.Variable";
            };

            return Variable;
        })();

        bindings.VariableList = (function() {

            /**
             * Properties of a VariableList.
             * @memberof revault.bindings
             * @interface IVariableList
             * @property {Array.<revault.bindings.IVariable>|null} [values] VariableList values
             */

            /**
             * Constructs a new VariableList.
             * @memberof revault.bindings
             * @classdesc Represents a VariableList.
             * @implements IVariableList
             * @constructor
             * @param {revault.bindings.IVariableList=} [properties] Properties to set
             */
            function VariableList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * VariableList values.
             * @member {Array.<revault.bindings.IVariable>} values
             * @memberof revault.bindings.VariableList
             * @instance
             */
            VariableList.prototype.values = $util.emptyArray;

            /**
             * Creates a new VariableList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.VariableList
             * @static
             * @param {revault.bindings.IVariableList=} [properties] Properties to set
             * @returns {revault.bindings.VariableList} VariableList instance
             */
            VariableList.create = function create(properties) {
                return new VariableList(properties);
            };

            /**
             * Encodes the specified VariableList message. Does not implicitly {@link revault.bindings.VariableList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.VariableList
             * @static
             * @param {revault.bindings.IVariableList} message VariableList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            VariableList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.Variable.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified VariableList message, length delimited. Does not implicitly {@link revault.bindings.VariableList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.VariableList
             * @static
             * @param {revault.bindings.IVariableList} message VariableList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            VariableList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a VariableList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.VariableList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.VariableList} VariableList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            VariableList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.VariableList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.Variable.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a VariableList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.VariableList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.VariableList} VariableList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            VariableList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a VariableList message.
             * @function verify
             * @memberof revault.bindings.VariableList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            VariableList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.Variable.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a VariableList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.VariableList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.VariableList} VariableList
             */
            VariableList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.VariableList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.VariableList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.VariableList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.VariableList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.VariableList.values: object expected");
                        message.values[i] = $root.revault.bindings.Variable.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a VariableList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.VariableList
             * @static
             * @param {revault.bindings.VariableList} message VariableList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            VariableList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.Variable.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this VariableList to JSON.
             * @function toJSON
             * @memberof revault.bindings.VariableList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            VariableList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for VariableList
             * @function getTypeUrl
             * @memberof revault.bindings.VariableList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            VariableList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.VariableList";
            };

            return VariableList;
        })();

        bindings.OptionalString = (function() {

            /**
             * Properties of an OptionalString.
             * @memberof revault.bindings
             * @interface IOptionalString
             * @property {boolean|null} [present] OptionalString present
             * @property {string|null} [value] OptionalString value
             */

            /**
             * Constructs a new OptionalString.
             * @memberof revault.bindings
             * @classdesc Represents an OptionalString.
             * @implements IOptionalString
             * @constructor
             * @param {revault.bindings.IOptionalString=} [properties] Properties to set
             */
            function OptionalString(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OptionalString present.
             * @member {boolean} present
             * @memberof revault.bindings.OptionalString
             * @instance
             */
            OptionalString.prototype.present = false;

            /**
             * OptionalString value.
             * @member {string} value
             * @memberof revault.bindings.OptionalString
             * @instance
             */
            OptionalString.prototype.value = "";

            /**
             * Creates a new OptionalString instance using the specified properties.
             * @function create
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {revault.bindings.IOptionalString=} [properties] Properties to set
             * @returns {revault.bindings.OptionalString} OptionalString instance
             */
            OptionalString.create = function create(properties) {
                return new OptionalString(properties);
            };

            /**
             * Encodes the specified OptionalString message. Does not implicitly {@link revault.bindings.OptionalString.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {revault.bindings.IOptionalString} message OptionalString message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalString.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.present != null && Object.hasOwnProperty.call(message, "present"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.present);
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.value);
                return writer;
            };

            /**
             * Encodes the specified OptionalString message, length delimited. Does not implicitly {@link revault.bindings.OptionalString.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {revault.bindings.IOptionalString} message OptionalString message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OptionalString.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OptionalString message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.OptionalString} OptionalString
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalString.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.OptionalString();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.present = reader.bool();
                            break;
                        }
                    case 2: {
                            message.value = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OptionalString message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.OptionalString} OptionalString
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OptionalString.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OptionalString message.
             * @function verify
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OptionalString.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.present != null && Object.hasOwnProperty.call(message, "present"))
                    if (typeof message.present !== "boolean")
                        return "present: boolean expected";
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    if (!$util.isString(message.value))
                        return "value: string expected";
                return null;
            };

            /**
             * Creates an OptionalString message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.OptionalString} OptionalString
             */
            OptionalString.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.OptionalString)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.OptionalString: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.OptionalString();
                if (object.present != null)
                    message.present = Boolean(object.present);
                if (object.value != null)
                    message.value = String(object.value);
                return message;
            };

            /**
             * Creates a plain object from an OptionalString message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {revault.bindings.OptionalString} message OptionalString
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OptionalString.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.present = false;
                    object.value = "";
                }
                if (message.present != null && Object.hasOwnProperty.call(message, "present"))
                    object.present = message.present;
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = message.value;
                return object;
            };

            /**
             * Converts this OptionalString to JSON.
             * @function toJSON
             * @memberof revault.bindings.OptionalString
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OptionalString.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for OptionalString
             * @function getTypeUrl
             * @memberof revault.bindings.OptionalString
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            OptionalString.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.OptionalString";
            };

            return OptionalString;
        })();

        bindings.OwnerInspection = (function() {

            /**
             * Properties of an OwnerInspection.
             * @memberof revault.bindings
             * @interface IOwnerInspection
             * @property {boolean|null} [signed] OwnerInspection signed
             * @property {string|null} [fingerprint] OwnerInspection fingerprint
             * @property {boolean|null} [hasFingerprint] OwnerInspection hasFingerprint
             */

            /**
             * Constructs a new OwnerInspection.
             * @memberof revault.bindings
             * @classdesc Represents an OwnerInspection.
             * @implements IOwnerInspection
             * @constructor
             * @param {revault.bindings.IOwnerInspection=} [properties] Properties to set
             */
            function OwnerInspection(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OwnerInspection signed.
             * @member {boolean} signed
             * @memberof revault.bindings.OwnerInspection
             * @instance
             */
            OwnerInspection.prototype.signed = false;

            /**
             * OwnerInspection fingerprint.
             * @member {string} fingerprint
             * @memberof revault.bindings.OwnerInspection
             * @instance
             */
            OwnerInspection.prototype.fingerprint = "";

            /**
             * OwnerInspection hasFingerprint.
             * @member {boolean} hasFingerprint
             * @memberof revault.bindings.OwnerInspection
             * @instance
             */
            OwnerInspection.prototype.hasFingerprint = false;

            /**
             * Creates a new OwnerInspection instance using the specified properties.
             * @function create
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {revault.bindings.IOwnerInspection=} [properties] Properties to set
             * @returns {revault.bindings.OwnerInspection} OwnerInspection instance
             */
            OwnerInspection.create = function create(properties) {
                return new OwnerInspection(properties);
            };

            /**
             * Encodes the specified OwnerInspection message. Does not implicitly {@link revault.bindings.OwnerInspection.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {revault.bindings.IOwnerInspection} message OwnerInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OwnerInspection.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.signed != null && Object.hasOwnProperty.call(message, "signed"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.signed);
                if (message.fingerprint != null && Object.hasOwnProperty.call(message, "fingerprint"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.fingerprint);
                if (message.hasFingerprint != null && Object.hasOwnProperty.call(message, "hasFingerprint"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.hasFingerprint);
                return writer;
            };

            /**
             * Encodes the specified OwnerInspection message, length delimited. Does not implicitly {@link revault.bindings.OwnerInspection.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {revault.bindings.IOwnerInspection} message OwnerInspection message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OwnerInspection.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OwnerInspection message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.OwnerInspection} OwnerInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OwnerInspection.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.OwnerInspection();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.signed = reader.bool();
                            break;
                        }
                    case 2: {
                            message.fingerprint = reader.string();
                            break;
                        }
                    case 3: {
                            message.hasFingerprint = reader.bool();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OwnerInspection message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.OwnerInspection} OwnerInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OwnerInspection.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OwnerInspection message.
             * @function verify
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OwnerInspection.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.signed != null && Object.hasOwnProperty.call(message, "signed"))
                    if (typeof message.signed !== "boolean")
                        return "signed: boolean expected";
                if (message.fingerprint != null && Object.hasOwnProperty.call(message, "fingerprint"))
                    if (!$util.isString(message.fingerprint))
                        return "fingerprint: string expected";
                if (message.hasFingerprint != null && Object.hasOwnProperty.call(message, "hasFingerprint"))
                    if (typeof message.hasFingerprint !== "boolean")
                        return "hasFingerprint: boolean expected";
                return null;
            };

            /**
             * Creates an OwnerInspection message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.OwnerInspection} OwnerInspection
             */
            OwnerInspection.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.OwnerInspection)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.OwnerInspection: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.OwnerInspection();
                if (object.signed != null)
                    message.signed = Boolean(object.signed);
                if (object.fingerprint != null)
                    message.fingerprint = String(object.fingerprint);
                if (object.hasFingerprint != null)
                    message.hasFingerprint = Boolean(object.hasFingerprint);
                return message;
            };

            /**
             * Creates a plain object from an OwnerInspection message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {revault.bindings.OwnerInspection} message OwnerInspection
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OwnerInspection.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.signed = false;
                    object.fingerprint = "";
                    object.hasFingerprint = false;
                }
                if (message.signed != null && Object.hasOwnProperty.call(message, "signed"))
                    object.signed = message.signed;
                if (message.fingerprint != null && Object.hasOwnProperty.call(message, "fingerprint"))
                    object.fingerprint = message.fingerprint;
                if (message.hasFingerprint != null && Object.hasOwnProperty.call(message, "hasFingerprint"))
                    object.hasFingerprint = message.hasFingerprint;
                return object;
            };

            /**
             * Converts this OwnerInspection to JSON.
             * @function toJSON
             * @memberof revault.bindings.OwnerInspection
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OwnerInspection.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for OwnerInspection
             * @function getTypeUrl
             * @memberof revault.bindings.OwnerInspection
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            OwnerInspection.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.OwnerInspection";
            };

            return OwnerInspection;
        })();

        bindings.Contact = (function() {

            /**
             * Properties of a Contact.
             * @memberof revault.bindings
             * @interface IContact
             * @property {string|null} [name] Contact name
             * @property {Uint8Array|null} [key] Contact key
             */

            /**
             * Constructs a new Contact.
             * @memberof revault.bindings
             * @classdesc Represents a Contact.
             * @implements IContact
             * @constructor
             * @param {revault.bindings.IContact=} [properties] Properties to set
             */
            function Contact(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Contact name.
             * @member {string} name
             * @memberof revault.bindings.Contact
             * @instance
             */
            Contact.prototype.name = "";

            /**
             * Contact key.
             * @member {Uint8Array} key
             * @memberof revault.bindings.Contact
             * @instance
             */
            Contact.prototype.key = $util.newBuffer([]);

            /**
             * Creates a new Contact instance using the specified properties.
             * @function create
             * @memberof revault.bindings.Contact
             * @static
             * @param {revault.bindings.IContact=} [properties] Properties to set
             * @returns {revault.bindings.Contact} Contact instance
             */
            Contact.create = function create(properties) {
                return new Contact(properties);
            };

            /**
             * Encodes the specified Contact message. Does not implicitly {@link revault.bindings.Contact.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.Contact
             * @static
             * @param {revault.bindings.IContact} message Contact message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Contact.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.key != null && Object.hasOwnProperty.call(message, "key"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.key);
                return writer;
            };

            /**
             * Encodes the specified Contact message, length delimited. Does not implicitly {@link revault.bindings.Contact.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.Contact
             * @static
             * @param {revault.bindings.IContact} message Contact message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Contact.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Contact message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.Contact
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.Contact} Contact
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Contact.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.Contact();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.name = reader.string();
                            break;
                        }
                    case 2: {
                            message.key = reader.bytes();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Contact message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.Contact
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.Contact} Contact
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Contact.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Contact message.
             * @function verify
             * @memberof revault.bindings.Contact
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Contact.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.key != null && Object.hasOwnProperty.call(message, "key"))
                    if (!(message.key && typeof message.key.length === "number" || $util.isString(message.key)))
                        return "key: buffer expected";
                return null;
            };

            /**
             * Creates a Contact message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.Contact
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.Contact} Contact
             */
            Contact.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.Contact)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.Contact: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.Contact();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.key != null)
                    if (typeof object.key === "string")
                        $util.base64.decode(object.key, message.key = $util.newBuffer($util.base64.length(object.key)), 0);
                    else if (object.key.length >= 0)
                        message.key = object.key;
                return message;
            };

            /**
             * Creates a plain object from a Contact message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.Contact
             * @static
             * @param {revault.bindings.Contact} message Contact
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Contact.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    if (options.bytes === String)
                        object.key = "";
                    else {
                        object.key = [];
                        if (options.bytes !== Array)
                            object.key = $util.newBuffer(object.key);
                    }
                }
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    object.name = message.name;
                if (message.key != null && Object.hasOwnProperty.call(message, "key"))
                    object.key = options.bytes === String ? $util.base64.encode(message.key, 0, message.key.length) : options.bytes === Array ? Array.prototype.slice.call(message.key) : message.key;
                return object;
            };

            /**
             * Converts this Contact to JSON.
             * @function toJSON
             * @memberof revault.bindings.Contact
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Contact.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for Contact
             * @function getTypeUrl
             * @memberof revault.bindings.Contact
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            Contact.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.Contact";
            };

            return Contact;
        })();

        bindings.ContactList = (function() {

            /**
             * Properties of a ContactList.
             * @memberof revault.bindings
             * @interface IContactList
             * @property {Array.<revault.bindings.IContact>|null} [values] ContactList values
             */

            /**
             * Constructs a new ContactList.
             * @memberof revault.bindings
             * @classdesc Represents a ContactList.
             * @implements IContactList
             * @constructor
             * @param {revault.bindings.IContactList=} [properties] Properties to set
             */
            function ContactList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ContactList values.
             * @member {Array.<revault.bindings.IContact>} values
             * @memberof revault.bindings.ContactList
             * @instance
             */
            ContactList.prototype.values = $util.emptyArray;

            /**
             * Creates a new ContactList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ContactList
             * @static
             * @param {revault.bindings.IContactList=} [properties] Properties to set
             * @returns {revault.bindings.ContactList} ContactList instance
             */
            ContactList.create = function create(properties) {
                return new ContactList(properties);
            };

            /**
             * Encodes the specified ContactList message. Does not implicitly {@link revault.bindings.ContactList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ContactList
             * @static
             * @param {revault.bindings.IContactList} message ContactList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ContactList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.Contact.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified ContactList message, length delimited. Does not implicitly {@link revault.bindings.ContactList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ContactList
             * @static
             * @param {revault.bindings.IContactList} message ContactList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ContactList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ContactList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ContactList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ContactList} ContactList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ContactList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ContactList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.Contact.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ContactList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ContactList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ContactList} ContactList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ContactList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ContactList message.
             * @function verify
             * @memberof revault.bindings.ContactList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ContactList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.Contact.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a ContactList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ContactList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ContactList} ContactList
             */
            ContactList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ContactList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ContactList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ContactList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.ContactList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.ContactList.values: object expected");
                        message.values[i] = $root.revault.bindings.Contact.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a ContactList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ContactList
             * @static
             * @param {revault.bindings.ContactList} message ContactList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ContactList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.Contact.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this ContactList to JSON.
             * @function toJSON
             * @memberof revault.bindings.ContactList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ContactList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ContactList
             * @function getTypeUrl
             * @memberof revault.bindings.ContactList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ContactList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ContactList";
            };

            return ContactList;
        })();

        bindings.ProfileHistoryList = (function() {

            /**
             * Properties of a ProfileHistoryList.
             * @memberof revault.bindings
             * @interface IProfileHistoryList
             * @property {Array.<revault.bindings.IProfileHistory>|null} [values] ProfileHistoryList values
             */

            /**
             * Constructs a new ProfileHistoryList.
             * @memberof revault.bindings
             * @classdesc Represents a ProfileHistoryList.
             * @implements IProfileHistoryList
             * @constructor
             * @param {revault.bindings.IProfileHistoryList=} [properties] Properties to set
             */
            function ProfileHistoryList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ProfileHistoryList values.
             * @member {Array.<revault.bindings.IProfileHistory>} values
             * @memberof revault.bindings.ProfileHistoryList
             * @instance
             */
            ProfileHistoryList.prototype.values = $util.emptyArray;

            /**
             * Creates a new ProfileHistoryList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {revault.bindings.IProfileHistoryList=} [properties] Properties to set
             * @returns {revault.bindings.ProfileHistoryList} ProfileHistoryList instance
             */
            ProfileHistoryList.create = function create(properties) {
                return new ProfileHistoryList(properties);
            };

            /**
             * Encodes the specified ProfileHistoryList message. Does not implicitly {@link revault.bindings.ProfileHistoryList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {revault.bindings.IProfileHistoryList} message ProfileHistoryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileHistoryList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.ProfileHistory.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified ProfileHistoryList message, length delimited. Does not implicitly {@link revault.bindings.ProfileHistoryList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {revault.bindings.IProfileHistoryList} message ProfileHistoryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ProfileHistoryList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ProfileHistoryList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ProfileHistoryList} ProfileHistoryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileHistoryList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ProfileHistoryList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.ProfileHistory.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ProfileHistoryList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ProfileHistoryList} ProfileHistoryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ProfileHistoryList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ProfileHistoryList message.
             * @function verify
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ProfileHistoryList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.ProfileHistory.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a ProfileHistoryList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ProfileHistoryList} ProfileHistoryList
             */
            ProfileHistoryList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ProfileHistoryList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ProfileHistoryList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ProfileHistoryList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.ProfileHistoryList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.ProfileHistoryList.values: object expected");
                        message.values[i] = $root.revault.bindings.ProfileHistory.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a ProfileHistoryList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {revault.bindings.ProfileHistoryList} message ProfileHistoryList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ProfileHistoryList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.ProfileHistory.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this ProfileHistoryList to JSON.
             * @function toJSON
             * @memberof revault.bindings.ProfileHistoryList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ProfileHistoryList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ProfileHistoryList
             * @function getTypeUrl
             * @memberof revault.bindings.ProfileHistoryList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ProfileHistoryList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ProfileHistoryList";
            };

            return ProfileHistoryList;
        })();

        bindings.AgentEntry = (function() {

            /**
             * Properties of an AgentEntry.
             * @memberof revault.bindings
             * @interface IAgentEntry
             * @property {string|null} [id] AgentEntry id
             * @property {string|null} [path] AgentEntry path
             */

            /**
             * Constructs a new AgentEntry.
             * @memberof revault.bindings
             * @classdesc Represents an AgentEntry.
             * @implements IAgentEntry
             * @constructor
             * @param {revault.bindings.IAgentEntry=} [properties] Properties to set
             */
            function AgentEntry(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AgentEntry id.
             * @member {string} id
             * @memberof revault.bindings.AgentEntry
             * @instance
             */
            AgentEntry.prototype.id = "";

            /**
             * AgentEntry path.
             * @member {string} path
             * @memberof revault.bindings.AgentEntry
             * @instance
             */
            AgentEntry.prototype.path = "";

            /**
             * Creates a new AgentEntry instance using the specified properties.
             * @function create
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {revault.bindings.IAgentEntry=} [properties] Properties to set
             * @returns {revault.bindings.AgentEntry} AgentEntry instance
             */
            AgentEntry.create = function create(properties) {
                return new AgentEntry(properties);
            };

            /**
             * Encodes the specified AgentEntry message. Does not implicitly {@link revault.bindings.AgentEntry.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {revault.bindings.IAgentEntry} message AgentEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentEntry.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.path);
                return writer;
            };

            /**
             * Encodes the specified AgentEntry message, length delimited. Does not implicitly {@link revault.bindings.AgentEntry.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {revault.bindings.IAgentEntry} message AgentEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentEntry.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AgentEntry message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.AgentEntry} AgentEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentEntry.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.AgentEntry();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.id = reader.string();
                            break;
                        }
                    case 2: {
                            message.path = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an AgentEntry message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.AgentEntry} AgentEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentEntry.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AgentEntry message.
             * @function verify
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AgentEntry.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    if (!$util.isString(message.id))
                        return "id: string expected";
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    if (!$util.isString(message.path))
                        return "path: string expected";
                return null;
            };

            /**
             * Creates an AgentEntry message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.AgentEntry} AgentEntry
             */
            AgentEntry.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.AgentEntry)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.AgentEntry: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.AgentEntry();
                if (object.id != null)
                    message.id = String(object.id);
                if (object.path != null)
                    message.path = String(object.path);
                return message;
            };

            /**
             * Creates a plain object from an AgentEntry message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {revault.bindings.AgentEntry} message AgentEntry
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AgentEntry.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.id = "";
                    object.path = "";
                }
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    object.id = message.id;
                if (message.path != null && Object.hasOwnProperty.call(message, "path"))
                    object.path = message.path;
                return object;
            };

            /**
             * Converts this AgentEntry to JSON.
             * @function toJSON
             * @memberof revault.bindings.AgentEntry
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AgentEntry.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for AgentEntry
             * @function getTypeUrl
             * @memberof revault.bindings.AgentEntry
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            AgentEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.AgentEntry";
            };

            return AgentEntry;
        })();

        bindings.AgentEntryList = (function() {

            /**
             * Properties of an AgentEntryList.
             * @memberof revault.bindings
             * @interface IAgentEntryList
             * @property {Array.<revault.bindings.IAgentEntry>|null} [values] AgentEntryList values
             */

            /**
             * Constructs a new AgentEntryList.
             * @memberof revault.bindings
             * @classdesc Represents an AgentEntryList.
             * @implements IAgentEntryList
             * @constructor
             * @param {revault.bindings.IAgentEntryList=} [properties] Properties to set
             */
            function AgentEntryList(properties) {
                this.values = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AgentEntryList values.
             * @member {Array.<revault.bindings.IAgentEntry>} values
             * @memberof revault.bindings.AgentEntryList
             * @instance
             */
            AgentEntryList.prototype.values = $util.emptyArray;

            /**
             * Creates a new AgentEntryList instance using the specified properties.
             * @function create
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {revault.bindings.IAgentEntryList=} [properties] Properties to set
             * @returns {revault.bindings.AgentEntryList} AgentEntryList instance
             */
            AgentEntryList.create = function create(properties) {
                return new AgentEntryList(properties);
            };

            /**
             * Encodes the specified AgentEntryList message. Does not implicitly {@link revault.bindings.AgentEntryList.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {revault.bindings.IAgentEntryList} message AgentEntryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentEntryList.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.values != null && message.values.length)
                    for (let i = 0; i < message.values.length; ++i)
                        $root.revault.bindings.AgentEntry.encode(message.values[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
                return writer;
            };

            /**
             * Encodes the specified AgentEntryList message, length delimited. Does not implicitly {@link revault.bindings.AgentEntryList.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {revault.bindings.IAgentEntryList} message AgentEntryList message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentEntryList.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AgentEntryList message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.AgentEntryList} AgentEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentEntryList.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.AgentEntryList();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            if (!(message.values && message.values.length))
                                message.values = [];
                            message.values.push($root.revault.bindings.AgentEntry.decode(reader, reader.uint32(), undefined, long + 1));
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an AgentEntryList message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.AgentEntryList} AgentEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentEntryList.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AgentEntryList message.
             * @function verify
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AgentEntryList.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.values != null && Object.hasOwnProperty.call(message, "values")) {
                    if (!Array.isArray(message.values))
                        return "values: array expected";
                    for (let i = 0; i < message.values.length; ++i) {
                        let error = $root.revault.bindings.AgentEntry.verify(message.values[i], long + 1);
                        if (error)
                            return "values." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an AgentEntryList message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.AgentEntryList} AgentEntryList
             */
            AgentEntryList.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.AgentEntryList)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.AgentEntryList: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.AgentEntryList();
                if (object.values) {
                    if (!Array.isArray(object.values))
                        throw TypeError(".revault.bindings.AgentEntryList.values: array expected");
                    message.values = [];
                    for (let i = 0; i < object.values.length; ++i) {
                        if (!$util.isObject(object.values[i]))
                            throw TypeError(".revault.bindings.AgentEntryList.values: object expected");
                        message.values[i] = $root.revault.bindings.AgentEntry.fromObject(object.values[i], long + 1);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an AgentEntryList message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {revault.bindings.AgentEntryList} message AgentEntryList
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AgentEntryList.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.arrays || options.defaults)
                    object.values = [];
                if (message.values && message.values.length) {
                    object.values = [];
                    for (let j = 0; j < message.values.length; ++j)
                        object.values[j] = $root.revault.bindings.AgentEntry.toObject(message.values[j], options, q + 1);
                }
                return object;
            };

            /**
             * Converts this AgentEntryList to JSON.
             * @function toJSON
             * @memberof revault.bindings.AgentEntryList
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AgentEntryList.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for AgentEntryList
             * @function getTypeUrl
             * @memberof revault.bindings.AgentEntryList
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            AgentEntryList.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.AgentEntryList";
            };

            return AgentEntryList;
        })();

        bindings.SleepSupport = (function() {

            /**
             * Properties of a SleepSupport.
             * @memberof revault.bindings
             * @interface ISleepSupport
             * @property {boolean|null} [suspendNotifications] SleepSupport suspendNotifications
             * @property {boolean|null} [sleepInhibition] SleepSupport sleepInhibition
             * @property {boolean|null} [supported] SleepSupport supported
             */

            /**
             * Constructs a new SleepSupport.
             * @memberof revault.bindings
             * @classdesc Represents a SleepSupport.
             * @implements ISleepSupport
             * @constructor
             * @param {revault.bindings.ISleepSupport=} [properties] Properties to set
             */
            function SleepSupport(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * SleepSupport suspendNotifications.
             * @member {boolean} suspendNotifications
             * @memberof revault.bindings.SleepSupport
             * @instance
             */
            SleepSupport.prototype.suspendNotifications = false;

            /**
             * SleepSupport sleepInhibition.
             * @member {boolean} sleepInhibition
             * @memberof revault.bindings.SleepSupport
             * @instance
             */
            SleepSupport.prototype.sleepInhibition = false;

            /**
             * SleepSupport supported.
             * @member {boolean} supported
             * @memberof revault.bindings.SleepSupport
             * @instance
             */
            SleepSupport.prototype.supported = false;

            /**
             * Creates a new SleepSupport instance using the specified properties.
             * @function create
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {revault.bindings.ISleepSupport=} [properties] Properties to set
             * @returns {revault.bindings.SleepSupport} SleepSupport instance
             */
            SleepSupport.create = function create(properties) {
                return new SleepSupport(properties);
            };

            /**
             * Encodes the specified SleepSupport message. Does not implicitly {@link revault.bindings.SleepSupport.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {revault.bindings.ISleepSupport} message SleepSupport message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SleepSupport.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.suspendNotifications != null && Object.hasOwnProperty.call(message, "suspendNotifications"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.suspendNotifications);
                if (message.sleepInhibition != null && Object.hasOwnProperty.call(message, "sleepInhibition"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.sleepInhibition);
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.supported);
                return writer;
            };

            /**
             * Encodes the specified SleepSupport message, length delimited. Does not implicitly {@link revault.bindings.SleepSupport.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {revault.bindings.ISleepSupport} message SleepSupport message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SleepSupport.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a SleepSupport message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.SleepSupport} SleepSupport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SleepSupport.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.SleepSupport();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.suspendNotifications = reader.bool();
                            break;
                        }
                    case 2: {
                            message.sleepInhibition = reader.bool();
                            break;
                        }
                    case 3: {
                            message.supported = reader.bool();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a SleepSupport message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.SleepSupport} SleepSupport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SleepSupport.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a SleepSupport message.
             * @function verify
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            SleepSupport.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.suspendNotifications != null && Object.hasOwnProperty.call(message, "suspendNotifications"))
                    if (typeof message.suspendNotifications !== "boolean")
                        return "suspendNotifications: boolean expected";
                if (message.sleepInhibition != null && Object.hasOwnProperty.call(message, "sleepInhibition"))
                    if (typeof message.sleepInhibition !== "boolean")
                        return "sleepInhibition: boolean expected";
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    if (typeof message.supported !== "boolean")
                        return "supported: boolean expected";
                return null;
            };

            /**
             * Creates a SleepSupport message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.SleepSupport} SleepSupport
             */
            SleepSupport.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.SleepSupport)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.SleepSupport: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.SleepSupport();
                if (object.suspendNotifications != null)
                    message.suspendNotifications = Boolean(object.suspendNotifications);
                if (object.sleepInhibition != null)
                    message.sleepInhibition = Boolean(object.sleepInhibition);
                if (object.supported != null)
                    message.supported = Boolean(object.supported);
                return message;
            };

            /**
             * Creates a plain object from a SleepSupport message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {revault.bindings.SleepSupport} message SleepSupport
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            SleepSupport.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.suspendNotifications = false;
                    object.sleepInhibition = false;
                    object.supported = false;
                }
                if (message.suspendNotifications != null && Object.hasOwnProperty.call(message, "suspendNotifications"))
                    object.suspendNotifications = message.suspendNotifications;
                if (message.sleepInhibition != null && Object.hasOwnProperty.call(message, "sleepInhibition"))
                    object.sleepInhibition = message.sleepInhibition;
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    object.supported = message.supported;
                return object;
            };

            /**
             * Converts this SleepSupport to JSON.
             * @function toJSON
             * @memberof revault.bindings.SleepSupport
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            SleepSupport.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for SleepSupport
             * @function getTypeUrl
             * @memberof revault.bindings.SleepSupport
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            SleepSupport.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.SleepSupport";
            };

            return SleepSupport;
        })();

        bindings.PlatformStatus = (function() {

            /**
             * Properties of a PlatformStatus.
             * @memberof revault.bindings
             * @interface IPlatformStatus
             * @property {boolean|null} [supported] PlatformStatus supported
             * @property {boolean|null} [disabled] PlatformStatus disabled
             * @property {string|null} [scope] PlatformStatus scope
             * @property {string|null} [backend] PlatformStatus backend
             * @property {string|null} [item] PlatformStatus item
             */

            /**
             * Constructs a new PlatformStatus.
             * @memberof revault.bindings
             * @classdesc Represents a PlatformStatus.
             * @implements IPlatformStatus
             * @constructor
             * @param {revault.bindings.IPlatformStatus=} [properties] Properties to set
             */
            function PlatformStatus(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PlatformStatus supported.
             * @member {boolean} supported
             * @memberof revault.bindings.PlatformStatus
             * @instance
             */
            PlatformStatus.prototype.supported = false;

            /**
             * PlatformStatus disabled.
             * @member {boolean} disabled
             * @memberof revault.bindings.PlatformStatus
             * @instance
             */
            PlatformStatus.prototype.disabled = false;

            /**
             * PlatformStatus scope.
             * @member {string} scope
             * @memberof revault.bindings.PlatformStatus
             * @instance
             */
            PlatformStatus.prototype.scope = "";

            /**
             * PlatformStatus backend.
             * @member {string} backend
             * @memberof revault.bindings.PlatformStatus
             * @instance
             */
            PlatformStatus.prototype.backend = "";

            /**
             * PlatformStatus item.
             * @member {string} item
             * @memberof revault.bindings.PlatformStatus
             * @instance
             */
            PlatformStatus.prototype.item = "";

            /**
             * Creates a new PlatformStatus instance using the specified properties.
             * @function create
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {revault.bindings.IPlatformStatus=} [properties] Properties to set
             * @returns {revault.bindings.PlatformStatus} PlatformStatus instance
             */
            PlatformStatus.create = function create(properties) {
                return new PlatformStatus(properties);
            };

            /**
             * Encodes the specified PlatformStatus message. Does not implicitly {@link revault.bindings.PlatformStatus.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {revault.bindings.IPlatformStatus} message PlatformStatus message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PlatformStatus.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.supported);
                if (message.disabled != null && Object.hasOwnProperty.call(message, "disabled"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.disabled);
                if (message.scope != null && Object.hasOwnProperty.call(message, "scope"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.scope);
                if (message.backend != null && Object.hasOwnProperty.call(message, "backend"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.backend);
                if (message.item != null && Object.hasOwnProperty.call(message, "item"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.item);
                return writer;
            };

            /**
             * Encodes the specified PlatformStatus message, length delimited. Does not implicitly {@link revault.bindings.PlatformStatus.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {revault.bindings.IPlatformStatus} message PlatformStatus message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PlatformStatus.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PlatformStatus message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.PlatformStatus} PlatformStatus
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PlatformStatus.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.PlatformStatus();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.supported = reader.bool();
                            break;
                        }
                    case 2: {
                            message.disabled = reader.bool();
                            break;
                        }
                    case 3: {
                            message.scope = reader.string();
                            break;
                        }
                    case 4: {
                            message.backend = reader.string();
                            break;
                        }
                    case 5: {
                            message.item = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PlatformStatus message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.PlatformStatus} PlatformStatus
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PlatformStatus.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PlatformStatus message.
             * @function verify
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PlatformStatus.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    if (typeof message.supported !== "boolean")
                        return "supported: boolean expected";
                if (message.disabled != null && Object.hasOwnProperty.call(message, "disabled"))
                    if (typeof message.disabled !== "boolean")
                        return "disabled: boolean expected";
                if (message.scope != null && Object.hasOwnProperty.call(message, "scope"))
                    if (!$util.isString(message.scope))
                        return "scope: string expected";
                if (message.backend != null && Object.hasOwnProperty.call(message, "backend"))
                    if (!$util.isString(message.backend))
                        return "backend: string expected";
                if (message.item != null && Object.hasOwnProperty.call(message, "item"))
                    if (!$util.isString(message.item))
                        return "item: string expected";
                return null;
            };

            /**
             * Creates a PlatformStatus message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.PlatformStatus} PlatformStatus
             */
            PlatformStatus.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.PlatformStatus)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.PlatformStatus: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.PlatformStatus();
                if (object.supported != null)
                    message.supported = Boolean(object.supported);
                if (object.disabled != null)
                    message.disabled = Boolean(object.disabled);
                if (object.scope != null)
                    message.scope = String(object.scope);
                if (object.backend != null)
                    message.backend = String(object.backend);
                if (object.item != null)
                    message.item = String(object.item);
                return message;
            };

            /**
             * Creates a plain object from a PlatformStatus message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {revault.bindings.PlatformStatus} message PlatformStatus
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PlatformStatus.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.supported = false;
                    object.disabled = false;
                    object.scope = "";
                    object.backend = "";
                    object.item = "";
                }
                if (message.supported != null && Object.hasOwnProperty.call(message, "supported"))
                    object.supported = message.supported;
                if (message.disabled != null && Object.hasOwnProperty.call(message, "disabled"))
                    object.disabled = message.disabled;
                if (message.scope != null && Object.hasOwnProperty.call(message, "scope"))
                    object.scope = message.scope;
                if (message.backend != null && Object.hasOwnProperty.call(message, "backend"))
                    object.backend = message.backend;
                if (message.item != null && Object.hasOwnProperty.call(message, "item"))
                    object.item = message.item;
                return object;
            };

            /**
             * Converts this PlatformStatus to JSON.
             * @function toJSON
             * @memberof revault.bindings.PlatformStatus
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PlatformStatus.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for PlatformStatus
             * @function getTypeUrl
             * @memberof revault.bindings.PlatformStatus
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            PlatformStatus.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.PlatformStatus";
            };

            return PlatformStatus;
        })();

        bindings.StringValue = (function() {

            /**
             * Properties of a StringValue.
             * @memberof revault.bindings
             * @interface IStringValue
             * @property {string|null} [value] StringValue value
             */

            /**
             * Constructs a new StringValue.
             * @memberof revault.bindings
             * @classdesc Represents a StringValue.
             * @implements IStringValue
             * @constructor
             * @param {revault.bindings.IStringValue=} [properties] Properties to set
             */
            function StringValue(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * StringValue value.
             * @member {string} value
             * @memberof revault.bindings.StringValue
             * @instance
             */
            StringValue.prototype.value = "";

            /**
             * Creates a new StringValue instance using the specified properties.
             * @function create
             * @memberof revault.bindings.StringValue
             * @static
             * @param {revault.bindings.IStringValue=} [properties] Properties to set
             * @returns {revault.bindings.StringValue} StringValue instance
             */
            StringValue.create = function create(properties) {
                return new StringValue(properties);
            };

            /**
             * Encodes the specified StringValue message. Does not implicitly {@link revault.bindings.StringValue.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.StringValue
             * @static
             * @param {revault.bindings.IStringValue} message StringValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StringValue.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.value);
                return writer;
            };

            /**
             * Encodes the specified StringValue message, length delimited. Does not implicitly {@link revault.bindings.StringValue.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.StringValue
             * @static
             * @param {revault.bindings.IStringValue} message StringValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StringValue.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a StringValue message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.StringValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.StringValue} StringValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StringValue.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.StringValue();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.value = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a StringValue message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.StringValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.StringValue} StringValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StringValue.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a StringValue message.
             * @function verify
             * @memberof revault.bindings.StringValue
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StringValue.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    if (!$util.isString(message.value))
                        return "value: string expected";
                return null;
            };

            /**
             * Creates a StringValue message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.StringValue
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.StringValue} StringValue
             */
            StringValue.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.StringValue)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.StringValue: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.StringValue();
                if (object.value != null)
                    message.value = String(object.value);
                return message;
            };

            /**
             * Creates a plain object from a StringValue message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.StringValue
             * @static
             * @param {revault.bindings.StringValue} message StringValue
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StringValue.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults)
                    object.value = "";
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    object.value = message.value;
                return object;
            };

            /**
             * Converts this StringValue to JSON.
             * @function toJSON
             * @memberof revault.bindings.StringValue
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StringValue.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for StringValue
             * @function getTypeUrl
             * @memberof revault.bindings.StringValue
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            StringValue.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.StringValue";
            };

            return StringValue;
        })();

        bindings.VaultBackupManifest = (function() {

            /**
             * Properties of a VaultBackupManifest.
             * @memberof revault.bindings
             * @interface IVaultBackupManifest
             * @property {number|null} [formatVersion] VaultBackupManifest formatVersion
             * @property {number|Long|null} [createdAtUnixMs] VaultBackupManifest createdAtUnixMs
             * @property {string|null} [vaultFileName] VaultBackupManifest vaultFileName
             * @property {number|Long|null} [vaultSize] VaultBackupManifest vaultSize
             * @property {string|null} [vaultSha256] VaultBackupManifest vaultSha256
             */

            /**
             * Constructs a new VaultBackupManifest.
             * @memberof revault.bindings
             * @classdesc Represents a VaultBackupManifest.
             * @implements IVaultBackupManifest
             * @constructor
             * @param {revault.bindings.IVaultBackupManifest=} [properties] Properties to set
             */
            function VaultBackupManifest(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * VaultBackupManifest formatVersion.
             * @member {number} formatVersion
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             */
            VaultBackupManifest.prototype.formatVersion = 0;

            /**
             * VaultBackupManifest createdAtUnixMs.
             * @member {number|Long} createdAtUnixMs
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             */
            VaultBackupManifest.prototype.createdAtUnixMs = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * VaultBackupManifest vaultFileName.
             * @member {string} vaultFileName
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             */
            VaultBackupManifest.prototype.vaultFileName = "";

            /**
             * VaultBackupManifest vaultSize.
             * @member {number|Long} vaultSize
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             */
            VaultBackupManifest.prototype.vaultSize = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * VaultBackupManifest vaultSha256.
             * @member {string} vaultSha256
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             */
            VaultBackupManifest.prototype.vaultSha256 = "";

            /**
             * Creates a new VaultBackupManifest instance using the specified properties.
             * @function create
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {revault.bindings.IVaultBackupManifest=} [properties] Properties to set
             * @returns {revault.bindings.VaultBackupManifest} VaultBackupManifest instance
             */
            VaultBackupManifest.create = function create(properties) {
                return new VaultBackupManifest(properties);
            };

            /**
             * Encodes the specified VaultBackupManifest message. Does not implicitly {@link revault.bindings.VaultBackupManifest.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {revault.bindings.IVaultBackupManifest} message VaultBackupManifest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            VaultBackupManifest.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.formatVersion != null && Object.hasOwnProperty.call(message, "formatVersion"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.formatVersion);
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.createdAtUnixMs);
                if (message.vaultFileName != null && Object.hasOwnProperty.call(message, "vaultFileName"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.vaultFileName);
                if (message.vaultSize != null && Object.hasOwnProperty.call(message, "vaultSize"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.vaultSize);
                if (message.vaultSha256 != null && Object.hasOwnProperty.call(message, "vaultSha256"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.vaultSha256);
                return writer;
            };

            /**
             * Encodes the specified VaultBackupManifest message, length delimited. Does not implicitly {@link revault.bindings.VaultBackupManifest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {revault.bindings.IVaultBackupManifest} message VaultBackupManifest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            VaultBackupManifest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a VaultBackupManifest message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.VaultBackupManifest} VaultBackupManifest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            VaultBackupManifest.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.VaultBackupManifest();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.formatVersion = reader.uint32();
                            break;
                        }
                    case 2: {
                            message.createdAtUnixMs = reader.uint64();
                            break;
                        }
                    case 3: {
                            message.vaultFileName = reader.string();
                            break;
                        }
                    case 4: {
                            message.vaultSize = reader.uint64();
                            break;
                        }
                    case 5: {
                            message.vaultSha256 = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a VaultBackupManifest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.VaultBackupManifest} VaultBackupManifest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            VaultBackupManifest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a VaultBackupManifest message.
             * @function verify
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            VaultBackupManifest.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.formatVersion != null && Object.hasOwnProperty.call(message, "formatVersion"))
                    if (!$util.isInteger(message.formatVersion))
                        return "formatVersion: integer expected";
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    if (!$util.isInteger(message.createdAtUnixMs) && !(message.createdAtUnixMs && $util.isInteger(message.createdAtUnixMs.low) && $util.isInteger(message.createdAtUnixMs.high)))
                        return "createdAtUnixMs: integer|Long expected";
                if (message.vaultFileName != null && Object.hasOwnProperty.call(message, "vaultFileName"))
                    if (!$util.isString(message.vaultFileName))
                        return "vaultFileName: string expected";
                if (message.vaultSize != null && Object.hasOwnProperty.call(message, "vaultSize"))
                    if (!$util.isInteger(message.vaultSize) && !(message.vaultSize && $util.isInteger(message.vaultSize.low) && $util.isInteger(message.vaultSize.high)))
                        return "vaultSize: integer|Long expected";
                if (message.vaultSha256 != null && Object.hasOwnProperty.call(message, "vaultSha256"))
                    if (!$util.isString(message.vaultSha256))
                        return "vaultSha256: string expected";
                return null;
            };

            /**
             * Creates a VaultBackupManifest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.VaultBackupManifest} VaultBackupManifest
             */
            VaultBackupManifest.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.VaultBackupManifest)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.VaultBackupManifest: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.VaultBackupManifest();
                if (object.formatVersion != null)
                    message.formatVersion = object.formatVersion >>> 0;
                if (object.createdAtUnixMs != null)
                    if ($util.Long)
                        message.createdAtUnixMs = $util.Long.fromValue(object.createdAtUnixMs, true);
                    else if (typeof object.createdAtUnixMs === "string")
                        message.createdAtUnixMs = parseInt(object.createdAtUnixMs, 10);
                    else if (typeof object.createdAtUnixMs === "number")
                        message.createdAtUnixMs = object.createdAtUnixMs;
                    else if (typeof object.createdAtUnixMs === "object")
                        message.createdAtUnixMs = new $util.LongBits(object.createdAtUnixMs.low >>> 0, object.createdAtUnixMs.high >>> 0).toNumber(true);
                if (object.vaultFileName != null)
                    message.vaultFileName = String(object.vaultFileName);
                if (object.vaultSize != null)
                    if ($util.Long)
                        message.vaultSize = $util.Long.fromValue(object.vaultSize, true);
                    else if (typeof object.vaultSize === "string")
                        message.vaultSize = parseInt(object.vaultSize, 10);
                    else if (typeof object.vaultSize === "number")
                        message.vaultSize = object.vaultSize;
                    else if (typeof object.vaultSize === "object")
                        message.vaultSize = new $util.LongBits(object.vaultSize.low >>> 0, object.vaultSize.high >>> 0).toNumber(true);
                if (object.vaultSha256 != null)
                    message.vaultSha256 = String(object.vaultSha256);
                return message;
            };

            /**
             * Creates a plain object from a VaultBackupManifest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {revault.bindings.VaultBackupManifest} message VaultBackupManifest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            VaultBackupManifest.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.formatVersion = 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.createdAtUnixMs = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.createdAtUnixMs = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.vaultFileName = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.vaultSize = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.vaultSize = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    object.vaultSha256 = "";
                }
                if (message.formatVersion != null && Object.hasOwnProperty.call(message, "formatVersion"))
                    object.formatVersion = message.formatVersion;
                if (message.createdAtUnixMs != null && Object.hasOwnProperty.call(message, "createdAtUnixMs"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.createdAtUnixMs = typeof message.createdAtUnixMs === "number" ? BigInt(message.createdAtUnixMs) : $util.Long.fromBits(message.createdAtUnixMs.low >>> 0, message.createdAtUnixMs.high >>> 0, true).toBigInt();
                    else if (typeof message.createdAtUnixMs === "number")
                        object.createdAtUnixMs = options.longs === String ? String(message.createdAtUnixMs) : message.createdAtUnixMs;
                    else
                        object.createdAtUnixMs = options.longs === String ? $util.Long.prototype.toString.call(message.createdAtUnixMs) : options.longs === Number ? new $util.LongBits(message.createdAtUnixMs.low >>> 0, message.createdAtUnixMs.high >>> 0).toNumber(true) : message.createdAtUnixMs;
                if (message.vaultFileName != null && Object.hasOwnProperty.call(message, "vaultFileName"))
                    object.vaultFileName = message.vaultFileName;
                if (message.vaultSize != null && Object.hasOwnProperty.call(message, "vaultSize"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.vaultSize = typeof message.vaultSize === "number" ? BigInt(message.vaultSize) : $util.Long.fromBits(message.vaultSize.low >>> 0, message.vaultSize.high >>> 0, true).toBigInt();
                    else if (typeof message.vaultSize === "number")
                        object.vaultSize = options.longs === String ? String(message.vaultSize) : message.vaultSize;
                    else
                        object.vaultSize = options.longs === String ? $util.Long.prototype.toString.call(message.vaultSize) : options.longs === Number ? new $util.LongBits(message.vaultSize.low >>> 0, message.vaultSize.high >>> 0).toNumber(true) : message.vaultSize;
                if (message.vaultSha256 != null && Object.hasOwnProperty.call(message, "vaultSha256"))
                    object.vaultSha256 = message.vaultSha256;
                return object;
            };

            /**
             * Converts this VaultBackupManifest to JSON.
             * @function toJSON
             * @memberof revault.bindings.VaultBackupManifest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            VaultBackupManifest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for VaultBackupManifest
             * @function getTypeUrl
             * @memberof revault.bindings.VaultBackupManifest
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            VaultBackupManifest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.VaultBackupManifest";
            };

            return VaultBackupManifest;
        })();

        bindings.ErrorDetails = (function() {

            /**
             * Properties of an ErrorDetails.
             * @memberof revault.bindings
             * @interface IErrorDetails
             * @property {string|null} [category] ErrorDetails category
             * @property {string|null} [artifactKind] ErrorDetails artifactKind
             * @property {number|null} [foundVersion] ErrorDetails foundVersion
             * @property {number|null} [supportedVersion] ErrorDetails supportedVersion
             * @property {string|null} [message] ErrorDetails message
             * @property {string|null} [guidance] ErrorDetails guidance
             */

            /**
             * Constructs a new ErrorDetails.
             * @memberof revault.bindings
             * @classdesc Represents an ErrorDetails.
             * @implements IErrorDetails
             * @constructor
             * @param {revault.bindings.IErrorDetails=} [properties] Properties to set
             */
            function ErrorDetails(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ErrorDetails category.
             * @member {string} category
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.category = "";

            /**
             * ErrorDetails artifactKind.
             * @member {string} artifactKind
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.artifactKind = "";

            /**
             * ErrorDetails foundVersion.
             * @member {number} foundVersion
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.foundVersion = 0;

            /**
             * ErrorDetails supportedVersion.
             * @member {number} supportedVersion
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.supportedVersion = 0;

            /**
             * ErrorDetails message.
             * @member {string} message
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.message = "";

            /**
             * ErrorDetails guidance.
             * @member {string} guidance
             * @memberof revault.bindings.ErrorDetails
             * @instance
             */
            ErrorDetails.prototype.guidance = "";

            /**
             * Creates a new ErrorDetails instance using the specified properties.
             * @function create
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {revault.bindings.IErrorDetails=} [properties] Properties to set
             * @returns {revault.bindings.ErrorDetails} ErrorDetails instance
             */
            ErrorDetails.create = function create(properties) {
                return new ErrorDetails(properties);
            };

            /**
             * Encodes the specified ErrorDetails message. Does not implicitly {@link revault.bindings.ErrorDetails.verify|verify} messages.
             * @function encode
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {revault.bindings.IErrorDetails} message ErrorDetails message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ErrorDetails.encode = function encode(message, writer, q) {
                if (!writer)
                    writer = $Writer.create();
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                if (message.category != null && Object.hasOwnProperty.call(message, "category"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.category);
                if (message.artifactKind != null && Object.hasOwnProperty.call(message, "artifactKind"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.artifactKind);
                if (message.foundVersion != null && Object.hasOwnProperty.call(message, "foundVersion"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.foundVersion);
                if (message.supportedVersion != null && Object.hasOwnProperty.call(message, "supportedVersion"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.supportedVersion);
                if (message.message != null && Object.hasOwnProperty.call(message, "message"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.message);
                if (message.guidance != null && Object.hasOwnProperty.call(message, "guidance"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.guidance);
                return writer;
            };

            /**
             * Encodes the specified ErrorDetails message, length delimited. Does not implicitly {@link revault.bindings.ErrorDetails.verify|verify} messages.
             * @function encodeDelimited
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {revault.bindings.IErrorDetails} message ErrorDetails message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ErrorDetails.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an ErrorDetails message from the specified reader or buffer.
             * @function decode
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {revault.bindings.ErrorDetails} ErrorDetails
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ErrorDetails.decode = function decode(reader, length, error, long) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (long === undefined)
                    long = 0;
                if (long > $Reader.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.revault.bindings.ErrorDetails();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    if (tag === error)
                        break;
                    switch (tag >>> 3) {
                    case 1: {
                            message.category = reader.string();
                            break;
                        }
                    case 2: {
                            message.artifactKind = reader.string();
                            break;
                        }
                    case 3: {
                            message.foundVersion = reader.uint32();
                            break;
                        }
                    case 4: {
                            message.supportedVersion = reader.uint32();
                            break;
                        }
                    case 5: {
                            message.message = reader.string();
                            break;
                        }
                    case 6: {
                            message.guidance = reader.string();
                            break;
                        }
                    default:
                        reader.skipType(tag & 7, long);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an ErrorDetails message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {revault.bindings.ErrorDetails} ErrorDetails
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ErrorDetails.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an ErrorDetails message.
             * @function verify
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ErrorDetails.verify = function verify(message, long) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    return "maximum nesting depth exceeded";
                if (message.category != null && Object.hasOwnProperty.call(message, "category"))
                    if (!$util.isString(message.category))
                        return "category: string expected";
                if (message.artifactKind != null && Object.hasOwnProperty.call(message, "artifactKind"))
                    if (!$util.isString(message.artifactKind))
                        return "artifactKind: string expected";
                if (message.foundVersion != null && Object.hasOwnProperty.call(message, "foundVersion"))
                    if (!$util.isInteger(message.foundVersion))
                        return "foundVersion: integer expected";
                if (message.supportedVersion != null && Object.hasOwnProperty.call(message, "supportedVersion"))
                    if (!$util.isInteger(message.supportedVersion))
                        return "supportedVersion: integer expected";
                if (message.message != null && Object.hasOwnProperty.call(message, "message"))
                    if (!$util.isString(message.message))
                        return "message: string expected";
                if (message.guidance != null && Object.hasOwnProperty.call(message, "guidance"))
                    if (!$util.isString(message.guidance))
                        return "guidance: string expected";
                return null;
            };

            /**
             * Creates an ErrorDetails message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {revault.bindings.ErrorDetails} ErrorDetails
             */
            ErrorDetails.fromObject = function fromObject(object, long) {
                if (object instanceof $root.revault.bindings.ErrorDetails)
                    return object;
                if (!$util.isObject(object))
                    throw TypeError(".revault.bindings.ErrorDetails: object expected");
                if (long === undefined)
                    long = 0;
                if (long > $util.recursionLimit)
                    throw Error("maximum nesting depth exceeded");
                let message = new $root.revault.bindings.ErrorDetails();
                if (object.category != null)
                    message.category = String(object.category);
                if (object.artifactKind != null)
                    message.artifactKind = String(object.artifactKind);
                if (object.foundVersion != null)
                    message.foundVersion = object.foundVersion >>> 0;
                if (object.supportedVersion != null)
                    message.supportedVersion = object.supportedVersion >>> 0;
                if (object.message != null)
                    message.message = String(object.message);
                if (object.guidance != null)
                    message.guidance = String(object.guidance);
                return message;
            };

            /**
             * Creates a plain object from an ErrorDetails message. Also converts values to other types if specified.
             * @function toObject
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {revault.bindings.ErrorDetails} message ErrorDetails
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ErrorDetails.toObject = function toObject(message, options, q) {
                if (!options)
                    options = {};
                if (q === undefined)
                    q = 0;
                if (q > $util.recursionLimit)
                    throw Error("max depth exceeded");
                let object = {};
                if (options.defaults) {
                    object.category = "";
                    object.artifactKind = "";
                    object.foundVersion = 0;
                    object.supportedVersion = 0;
                    object.message = "";
                    object.guidance = "";
                }
                if (message.category != null && Object.hasOwnProperty.call(message, "category"))
                    object.category = message.category;
                if (message.artifactKind != null && Object.hasOwnProperty.call(message, "artifactKind"))
                    object.artifactKind = message.artifactKind;
                if (message.foundVersion != null && Object.hasOwnProperty.call(message, "foundVersion"))
                    object.foundVersion = message.foundVersion;
                if (message.supportedVersion != null && Object.hasOwnProperty.call(message, "supportedVersion"))
                    object.supportedVersion = message.supportedVersion;
                if (message.message != null && Object.hasOwnProperty.call(message, "message"))
                    object.message = message.message;
                if (message.guidance != null && Object.hasOwnProperty.call(message, "guidance"))
                    object.guidance = message.guidance;
                return object;
            };

            /**
             * Converts this ErrorDetails to JSON.
             * @function toJSON
             * @memberof revault.bindings.ErrorDetails
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ErrorDetails.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for ErrorDetails
             * @function getTypeUrl
             * @memberof revault.bindings.ErrorDetails
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            ErrorDetails.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/revault.bindings.ErrorDetails";
            };

            return ErrorDetails;
        })();

        return bindings;
    })();

    return revault;
})();

export { $root as default };
