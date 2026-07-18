import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace revault. */
export namespace revault {

    /** Namespace bindings. */
    namespace bindings {

        /** Properties of a LockboxEntry. */
        interface ILockboxEntry {

            /** LockboxEntry path */
            path?: (string|null);

            /** LockboxEntry kind */
            kind?: (revault.bindings.LockboxEntry.Kind|null);

            /** LockboxEntry length */
            length?: (number|Long|null);

            /** LockboxEntry permissions */
            permissions?: (number|null);
        }

        /** Represents a LockboxEntry. */
        class LockboxEntry implements ILockboxEntry {

            /**
             * Constructs a new LockboxEntry.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.ILockboxEntry);

            /** LockboxEntry path. */
            public path: string;

            /** LockboxEntry kind. */
            public kind: revault.bindings.LockboxEntry.Kind;

            /** LockboxEntry length. */
            public length: (number|Long);

            /** LockboxEntry permissions. */
            public permissions: number;

            /**
             * Creates a new LockboxEntry instance using the specified properties.
             * @param [properties] Properties to set
             * @returns LockboxEntry instance
             */
            public static create(properties?: revault.bindings.ILockboxEntry): revault.bindings.LockboxEntry;

            /**
             * Encodes the specified LockboxEntry message. Does not implicitly {@link revault.bindings.LockboxEntry.verify|verify} messages.
             * @param message LockboxEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.ILockboxEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified LockboxEntry message, length delimited. Does not implicitly {@link revault.bindings.LockboxEntry.verify|verify} messages.
             * @param message LockboxEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.ILockboxEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a LockboxEntry message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns LockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.LockboxEntry;

            /**
             * Decodes a LockboxEntry message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns LockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.LockboxEntry;

            /**
             * Verifies a LockboxEntry message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a LockboxEntry message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns LockboxEntry
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.LockboxEntry;

            /**
             * Creates a plain object from a LockboxEntry message. Also converts values to other types if specified.
             * @param message LockboxEntry
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.LockboxEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this LockboxEntry to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for LockboxEntry
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        namespace LockboxEntry {

            /** Kind enum. */
            enum Kind {
                KIND_UNSPECIFIED = 0,
                FILE = 1,
                SYMLINK = 2,
                DIRECTORY = 3
            }
        }

        /** Properties of a LockboxEntryList. */
        interface ILockboxEntryList {

            /** LockboxEntryList entries */
            entries?: (revault.bindings.ILockboxEntry[]|null);
        }

        /** Represents a LockboxEntryList. */
        class LockboxEntryList implements ILockboxEntryList {

            /**
             * Constructs a new LockboxEntryList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.ILockboxEntryList);

            /** LockboxEntryList entries. */
            public entries: revault.bindings.ILockboxEntry[];

            /**
             * Creates a new LockboxEntryList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns LockboxEntryList instance
             */
            public static create(properties?: revault.bindings.ILockboxEntryList): revault.bindings.LockboxEntryList;

            /**
             * Encodes the specified LockboxEntryList message. Does not implicitly {@link revault.bindings.LockboxEntryList.verify|verify} messages.
             * @param message LockboxEntryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.ILockboxEntryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified LockboxEntryList message, length delimited. Does not implicitly {@link revault.bindings.LockboxEntryList.verify|verify} messages.
             * @param message LockboxEntryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.ILockboxEntryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a LockboxEntryList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns LockboxEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.LockboxEntryList;

            /**
             * Decodes a LockboxEntryList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns LockboxEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.LockboxEntryList;

            /**
             * Verifies a LockboxEntryList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a LockboxEntryList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns LockboxEntryList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.LockboxEntryList;

            /**
             * Creates a plain object from a LockboxEntryList message. Also converts values to other types if specified.
             * @param message LockboxEntryList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.LockboxEntryList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this LockboxEntryList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for LockboxEntryList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an OptionalLockboxEntry. */
        interface IOptionalLockboxEntry {

            /** OptionalLockboxEntry value */
            value?: (revault.bindings.ILockboxEntry|null);
        }

        /** Represents an OptionalLockboxEntry. */
        class OptionalLockboxEntry implements IOptionalLockboxEntry {

            /**
             * Constructs a new OptionalLockboxEntry.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IOptionalLockboxEntry);

            /** OptionalLockboxEntry value. */
            public value?: (revault.bindings.ILockboxEntry|null);

            /**
             * Creates a new OptionalLockboxEntry instance using the specified properties.
             * @param [properties] Properties to set
             * @returns OptionalLockboxEntry instance
             */
            public static create(properties?: revault.bindings.IOptionalLockboxEntry): revault.bindings.OptionalLockboxEntry;

            /**
             * Encodes the specified OptionalLockboxEntry message. Does not implicitly {@link revault.bindings.OptionalLockboxEntry.verify|verify} messages.
             * @param message OptionalLockboxEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IOptionalLockboxEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified OptionalLockboxEntry message, length delimited. Does not implicitly {@link revault.bindings.OptionalLockboxEntry.verify|verify} messages.
             * @param message OptionalLockboxEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IOptionalLockboxEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an OptionalLockboxEntry message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns OptionalLockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.OptionalLockboxEntry;

            /**
             * Decodes an OptionalLockboxEntry message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns OptionalLockboxEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.OptionalLockboxEntry;

            /**
             * Verifies an OptionalLockboxEntry message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an OptionalLockboxEntry message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns OptionalLockboxEntry
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.OptionalLockboxEntry;

            /**
             * Creates a plain object from an OptionalLockboxEntry message. Also converts values to other types if specified.
             * @param message OptionalLockboxEntry
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.OptionalLockboxEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this OptionalLockboxEntry to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for OptionalLockboxEntry
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a StringList. */
        interface IStringList {

            /** StringList values */
            values?: (string[]|null);
        }

        /** Represents a StringList. */
        class StringList implements IStringList {

            /**
             * Constructs a new StringList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IStringList);

            /** StringList values. */
            public values: string[];

            /**
             * Creates a new StringList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns StringList instance
             */
            public static create(properties?: revault.bindings.IStringList): revault.bindings.StringList;

            /**
             * Encodes the specified StringList message. Does not implicitly {@link revault.bindings.StringList.verify|verify} messages.
             * @param message StringList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IStringList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified StringList message, length delimited. Does not implicitly {@link revault.bindings.StringList.verify|verify} messages.
             * @param message StringList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IStringList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a StringList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns StringList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.StringList;

            /**
             * Decodes a StringList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns StringList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.StringList;

            /**
             * Verifies a StringList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a StringList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns StringList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.StringList;

            /**
             * Creates a plain object from a StringList message. Also converts values to other types if specified.
             * @param message StringList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.StringList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this StringList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for StringList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PathMove. */
        interface IPathMove {

            /** PathMove source */
            source?: (string|null);

            /** PathMove destination */
            destination?: (string|null);
        }

        /** Represents a PathMove. */
        class PathMove implements IPathMove {

            /**
             * Constructs a new PathMove.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPathMove);

            /** PathMove source. */
            public source: string;

            /** PathMove destination. */
            public destination: string;

            /**
             * Creates a new PathMove instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PathMove instance
             */
            public static create(properties?: revault.bindings.IPathMove): revault.bindings.PathMove;

            /**
             * Encodes the specified PathMove message. Does not implicitly {@link revault.bindings.PathMove.verify|verify} messages.
             * @param message PathMove message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPathMove, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PathMove message, length delimited. Does not implicitly {@link revault.bindings.PathMove.verify|verify} messages.
             * @param message PathMove message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPathMove, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PathMove message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PathMove
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PathMove;

            /**
             * Decodes a PathMove message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PathMove
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PathMove;

            /**
             * Verifies a PathMove message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PathMove message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PathMove
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PathMove;

            /**
             * Creates a plain object from a PathMove message. Also converts values to other types if specified.
             * @param message PathMove
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PathMove, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PathMove to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PathMove
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PathMoveList. */
        interface IPathMoveList {

            /** PathMoveList values */
            values?: (revault.bindings.IPathMove[]|null);
        }

        /** Represents a PathMoveList. */
        class PathMoveList implements IPathMoveList {

            /**
             * Constructs a new PathMoveList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPathMoveList);

            /** PathMoveList values. */
            public values: revault.bindings.IPathMove[];

            /**
             * Creates a new PathMoveList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PathMoveList instance
             */
            public static create(properties?: revault.bindings.IPathMoveList): revault.bindings.PathMoveList;

            /**
             * Encodes the specified PathMoveList message. Does not implicitly {@link revault.bindings.PathMoveList.verify|verify} messages.
             * @param message PathMoveList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPathMoveList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PathMoveList message, length delimited. Does not implicitly {@link revault.bindings.PathMoveList.verify|verify} messages.
             * @param message PathMoveList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPathMoveList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PathMoveList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PathMoveList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PathMoveList;

            /**
             * Decodes a PathMoveList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PathMoveList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PathMoveList;

            /**
             * Verifies a PathMoveList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PathMoveList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PathMoveList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PathMoveList;

            /**
             * Creates a plain object from a PathMoveList message. Also converts values to other types if specified.
             * @param message PathMoveList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PathMoveList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PathMoveList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PathMoveList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a ByteList. */
        interface IByteList {

            /** ByteList values */
            values?: (Uint8Array[]|null);
        }

        /** Represents a ByteList. */
        class ByteList implements IByteList {

            /**
             * Constructs a new ByteList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IByteList);

            /** ByteList values. */
            public values: Uint8Array[];

            /**
             * Creates a new ByteList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ByteList instance
             */
            public static create(properties?: revault.bindings.IByteList): revault.bindings.ByteList;

            /**
             * Encodes the specified ByteList message. Does not implicitly {@link revault.bindings.ByteList.verify|verify} messages.
             * @param message ByteList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IByteList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ByteList message, length delimited. Does not implicitly {@link revault.bindings.ByteList.verify|verify} messages.
             * @param message ByteList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IByteList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ByteList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ByteList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ByteList;

            /**
             * Decodes a ByteList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ByteList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ByteList;

            /**
             * Verifies a ByteList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ByteList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ByteList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ByteList;

            /**
             * Creates a plain object from a ByteList message. Also converts values to other types if specified.
             * @param message ByteList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ByteList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ByteList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ByteList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormField. */
        interface IFormField {

            /** FormField id */
            id?: (string|null);

            /** FormField label */
            label?: (string|null);

            /** FormField kind */
            kind?: (string|null);

            /** FormField required */
            required?: (boolean|null);
        }

        /** Represents a FormField. */
        class FormField implements IFormField {

            /**
             * Constructs a new FormField.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormField);

            /** FormField id. */
            public id: string;

            /** FormField label. */
            public label: string;

            /** FormField kind. */
            public kind: string;

            /** FormField required. */
            public required: boolean;

            /**
             * Creates a new FormField instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormField instance
             */
            public static create(properties?: revault.bindings.IFormField): revault.bindings.FormField;

            /**
             * Encodes the specified FormField message. Does not implicitly {@link revault.bindings.FormField.verify|verify} messages.
             * @param message FormField message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormField, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormField message, length delimited. Does not implicitly {@link revault.bindings.FormField.verify|verify} messages.
             * @param message FormField message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormField, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormField message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormField;

            /**
             * Decodes a FormField message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormField;

            /**
             * Verifies a FormField message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormField message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormField
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormField;

            /**
             * Creates a plain object from a FormField message. Also converts values to other types if specified.
             * @param message FormField
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormField, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormField to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormField
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormFieldList. */
        interface IFormFieldList {

            /** FormFieldList values */
            values?: (revault.bindings.IFormField[]|null);
        }

        /** Represents a FormFieldList. */
        class FormFieldList implements IFormFieldList {

            /**
             * Constructs a new FormFieldList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormFieldList);

            /** FormFieldList values. */
            public values: revault.bindings.IFormField[];

            /**
             * Creates a new FormFieldList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormFieldList instance
             */
            public static create(properties?: revault.bindings.IFormFieldList): revault.bindings.FormFieldList;

            /**
             * Encodes the specified FormFieldList message. Does not implicitly {@link revault.bindings.FormFieldList.verify|verify} messages.
             * @param message FormFieldList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormFieldList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormFieldList message, length delimited. Does not implicitly {@link revault.bindings.FormFieldList.verify|verify} messages.
             * @param message FormFieldList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormFieldList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormFieldList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormFieldList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormFieldList;

            /**
             * Decodes a FormFieldList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormFieldList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormFieldList;

            /**
             * Verifies a FormFieldList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormFieldList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormFieldList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormFieldList;

            /**
             * Creates a plain object from a FormFieldList message. Also converts values to other types if specified.
             * @param message FormFieldList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormFieldList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormFieldList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormFieldList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormDefinition. */
        interface IFormDefinition {

            /** FormDefinition typeId */
            typeId?: (string|null);

            /** FormDefinition alias */
            alias?: (string|null);

            /** FormDefinition revision */
            revision?: (number|null);

            /** FormDefinition name */
            name?: (string|null);

            /** FormDefinition description */
            description?: (string|null);

            /** FormDefinition fields */
            fields?: (revault.bindings.IFormField[]|null);
        }

        /** Represents a FormDefinition. */
        class FormDefinition implements IFormDefinition {

            /**
             * Constructs a new FormDefinition.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormDefinition);

            /** FormDefinition typeId. */
            public typeId: string;

            /** FormDefinition alias. */
            public alias: string;

            /** FormDefinition revision. */
            public revision: number;

            /** FormDefinition name. */
            public name: string;

            /** FormDefinition description. */
            public description: string;

            /** FormDefinition fields. */
            public fields: revault.bindings.IFormField[];

            /**
             * Creates a new FormDefinition instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormDefinition instance
             */
            public static create(properties?: revault.bindings.IFormDefinition): revault.bindings.FormDefinition;

            /**
             * Encodes the specified FormDefinition message. Does not implicitly {@link revault.bindings.FormDefinition.verify|verify} messages.
             * @param message FormDefinition message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormDefinition, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormDefinition message, length delimited. Does not implicitly {@link revault.bindings.FormDefinition.verify|verify} messages.
             * @param message FormDefinition message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormDefinition, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormDefinition message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormDefinition
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormDefinition;

            /**
             * Decodes a FormDefinition message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormDefinition
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormDefinition;

            /**
             * Verifies a FormDefinition message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormDefinition message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormDefinition
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormDefinition;

            /**
             * Creates a plain object from a FormDefinition message. Also converts values to other types if specified.
             * @param message FormDefinition
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormDefinition, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormDefinition to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormDefinition
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormDefinitionList. */
        interface IFormDefinitionList {

            /** FormDefinitionList values */
            values?: (revault.bindings.IFormDefinition[]|null);
        }

        /** Represents a FormDefinitionList. */
        class FormDefinitionList implements IFormDefinitionList {

            /**
             * Constructs a new FormDefinitionList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormDefinitionList);

            /** FormDefinitionList values. */
            public values: revault.bindings.IFormDefinition[];

            /**
             * Creates a new FormDefinitionList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormDefinitionList instance
             */
            public static create(properties?: revault.bindings.IFormDefinitionList): revault.bindings.FormDefinitionList;

            /**
             * Encodes the specified FormDefinitionList message. Does not implicitly {@link revault.bindings.FormDefinitionList.verify|verify} messages.
             * @param message FormDefinitionList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormDefinitionList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormDefinitionList message, length delimited. Does not implicitly {@link revault.bindings.FormDefinitionList.verify|verify} messages.
             * @param message FormDefinitionList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormDefinitionList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormDefinitionList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormDefinitionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormDefinitionList;

            /**
             * Decodes a FormDefinitionList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormDefinitionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormDefinitionList;

            /**
             * Verifies a FormDefinitionList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormDefinitionList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormDefinitionList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormDefinitionList;

            /**
             * Creates a plain object from a FormDefinitionList message. Also converts values to other types if specified.
             * @param message FormDefinitionList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormDefinitionList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormDefinitionList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormDefinitionList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormValue. */
        interface IFormValue {

            /** FormValue fieldId */
            fieldId?: (string|null);

            /** FormValue label */
            label?: (string|null);

            /** FormValue kind */
            kind?: (string|null);

            /** FormValue value */
            value?: (string|null);

            /** FormValue secret */
            secret?: (boolean|null);
        }

        /** Represents a FormValue. */
        class FormValue implements IFormValue {

            /**
             * Constructs a new FormValue.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormValue);

            /** FormValue fieldId. */
            public fieldId: string;

            /** FormValue label. */
            public label: string;

            /** FormValue kind. */
            public kind: string;

            /** FormValue value. */
            public value: string;

            /** FormValue secret. */
            public secret: boolean;

            /**
             * Creates a new FormValue instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormValue instance
             */
            public static create(properties?: revault.bindings.IFormValue): revault.bindings.FormValue;

            /**
             * Encodes the specified FormValue message. Does not implicitly {@link revault.bindings.FormValue.verify|verify} messages.
             * @param message FormValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormValue message, length delimited. Does not implicitly {@link revault.bindings.FormValue.verify|verify} messages.
             * @param message FormValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormValue message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormValue;

            /**
             * Decodes a FormValue message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormValue;

            /**
             * Verifies a FormValue message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormValue message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormValue
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormValue;

            /**
             * Creates a plain object from a FormValue message. Also converts values to other types if specified.
             * @param message FormValue
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormValue, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormValue to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormValue
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormRecord. */
        interface IFormRecord {

            /** FormRecord path */
            path?: (string|null);

            /** FormRecord name */
            name?: (string|null);

            /** FormRecord typeId */
            typeId?: (string|null);

            /** FormRecord definitionAlias */
            definitionAlias?: (string|null);

            /** FormRecord definitionRevision */
            definitionRevision?: (number|null);

            /** FormRecord values */
            values?: (revault.bindings.IFormValue[]|null);
        }

        /** Represents a FormRecord. */
        class FormRecord implements IFormRecord {

            /**
             * Constructs a new FormRecord.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormRecord);

            /** FormRecord path. */
            public path: string;

            /** FormRecord name. */
            public name: string;

            /** FormRecord typeId. */
            public typeId: string;

            /** FormRecord definitionAlias. */
            public definitionAlias: string;

            /** FormRecord definitionRevision. */
            public definitionRevision: number;

            /** FormRecord values. */
            public values: revault.bindings.IFormValue[];

            /**
             * Creates a new FormRecord instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormRecord instance
             */
            public static create(properties?: revault.bindings.IFormRecord): revault.bindings.FormRecord;

            /**
             * Encodes the specified FormRecord message. Does not implicitly {@link revault.bindings.FormRecord.verify|verify} messages.
             * @param message FormRecord message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormRecord, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormRecord message, length delimited. Does not implicitly {@link revault.bindings.FormRecord.verify|verify} messages.
             * @param message FormRecord message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormRecord, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormRecord message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormRecord;

            /**
             * Decodes a FormRecord message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormRecord;

            /**
             * Verifies a FormRecord message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormRecord message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormRecord
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormRecord;

            /**
             * Creates a plain object from a FormRecord message. Also converts values to other types if specified.
             * @param message FormRecord
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormRecord, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormRecord to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormRecord
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FormRecordList. */
        interface IFormRecordList {

            /** FormRecordList values */
            values?: (revault.bindings.IFormRecord[]|null);
        }

        /** Represents a FormRecordList. */
        class FormRecordList implements IFormRecordList {

            /**
             * Constructs a new FormRecordList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFormRecordList);

            /** FormRecordList values. */
            public values: revault.bindings.IFormRecord[];

            /**
             * Creates a new FormRecordList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FormRecordList instance
             */
            public static create(properties?: revault.bindings.IFormRecordList): revault.bindings.FormRecordList;

            /**
             * Encodes the specified FormRecordList message. Does not implicitly {@link revault.bindings.FormRecordList.verify|verify} messages.
             * @param message FormRecordList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFormRecordList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FormRecordList message, length delimited. Does not implicitly {@link revault.bindings.FormRecordList.verify|verify} messages.
             * @param message FormRecordList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFormRecordList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FormRecordList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FormRecordList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FormRecordList;

            /**
             * Decodes a FormRecordList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FormRecordList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FormRecordList;

            /**
             * Verifies a FormRecordList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FormRecordList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FormRecordList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FormRecordList;

            /**
             * Creates a plain object from a FormRecordList message. Also converts values to other types if specified.
             * @param message FormRecordList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FormRecordList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FormRecordList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FormRecordList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an OptionalFormRecord. */
        interface IOptionalFormRecord {

            /** OptionalFormRecord value */
            value?: (revault.bindings.IFormRecord|null);
        }

        /** Represents an OptionalFormRecord. */
        class OptionalFormRecord implements IOptionalFormRecord {

            /**
             * Constructs a new OptionalFormRecord.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IOptionalFormRecord);

            /** OptionalFormRecord value. */
            public value?: (revault.bindings.IFormRecord|null);

            /**
             * Creates a new OptionalFormRecord instance using the specified properties.
             * @param [properties] Properties to set
             * @returns OptionalFormRecord instance
             */
            public static create(properties?: revault.bindings.IOptionalFormRecord): revault.bindings.OptionalFormRecord;

            /**
             * Encodes the specified OptionalFormRecord message. Does not implicitly {@link revault.bindings.OptionalFormRecord.verify|verify} messages.
             * @param message OptionalFormRecord message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IOptionalFormRecord, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified OptionalFormRecord message, length delimited. Does not implicitly {@link revault.bindings.OptionalFormRecord.verify|verify} messages.
             * @param message OptionalFormRecord message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IOptionalFormRecord, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an OptionalFormRecord message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns OptionalFormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.OptionalFormRecord;

            /**
             * Decodes an OptionalFormRecord message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns OptionalFormRecord
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.OptionalFormRecord;

            /**
             * Verifies an OptionalFormRecord message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an OptionalFormRecord message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns OptionalFormRecord
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.OptionalFormRecord;

            /**
             * Creates a plain object from an OptionalFormRecord message. Also converts values to other types if specified.
             * @param message OptionalFormRecord
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.OptionalFormRecord, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this OptionalFormRecord to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for OptionalFormRecord
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an OptionalFormValue. */
        interface IOptionalFormValue {

            /** OptionalFormValue value */
            value?: (revault.bindings.IFormValue|null);
        }

        /** Represents an OptionalFormValue. */
        class OptionalFormValue implements IOptionalFormValue {

            /**
             * Constructs a new OptionalFormValue.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IOptionalFormValue);

            /** OptionalFormValue value. */
            public value?: (revault.bindings.IFormValue|null);

            /**
             * Creates a new OptionalFormValue instance using the specified properties.
             * @param [properties] Properties to set
             * @returns OptionalFormValue instance
             */
            public static create(properties?: revault.bindings.IOptionalFormValue): revault.bindings.OptionalFormValue;

            /**
             * Encodes the specified OptionalFormValue message. Does not implicitly {@link revault.bindings.OptionalFormValue.verify|verify} messages.
             * @param message OptionalFormValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IOptionalFormValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified OptionalFormValue message, length delimited. Does not implicitly {@link revault.bindings.OptionalFormValue.verify|verify} messages.
             * @param message OptionalFormValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IOptionalFormValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an OptionalFormValue message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns OptionalFormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.OptionalFormValue;

            /**
             * Decodes an OptionalFormValue message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns OptionalFormValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.OptionalFormValue;

            /**
             * Verifies an OptionalFormValue message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an OptionalFormValue message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns OptionalFormValue
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.OptionalFormValue;

            /**
             * Creates a plain object from an OptionalFormValue message. Also converts values to other types if specified.
             * @param message OptionalFormValue
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.OptionalFormValue, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this OptionalFormValue to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for OptionalFormValue
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a RecoveryReport. */
        interface IRecoveryReport {

            /** RecoveryReport intactFiles */
            intactFiles?: (revault.bindings.ILockboxEntry[]|null);

            /** RecoveryReport intactFileCount */
            intactFileCount?: (number|Long|null);

            /** RecoveryReport partialFiles */
            partialFiles?: (number|Long|null);

            /** RecoveryReport corruptRecords */
            corruptRecords?: (number|Long|null);

            /** RecoveryReport tocRecovered */
            tocRecovered?: (boolean|null);

            /** RecoveryReport variablesRecovered */
            variablesRecovered?: (boolean|null);

            /** RecoveryReport variableCount */
            variableCount?: (number|Long|null);

            /** RecoveryReport formsRecovered */
            formsRecovered?: (boolean|null);

            /** RecoveryReport formDefinitionCount */
            formDefinitionCount?: (number|Long|null);

            /** RecoveryReport formRecordCount */
            formRecordCount?: (number|Long|null);
        }

        /** Represents a RecoveryReport. */
        class RecoveryReport implements IRecoveryReport {

            /**
             * Constructs a new RecoveryReport.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IRecoveryReport);

            /** RecoveryReport intactFiles. */
            public intactFiles: revault.bindings.ILockboxEntry[];

            /** RecoveryReport intactFileCount. */
            public intactFileCount: (number|Long);

            /** RecoveryReport partialFiles. */
            public partialFiles: (number|Long);

            /** RecoveryReport corruptRecords. */
            public corruptRecords: (number|Long);

            /** RecoveryReport tocRecovered. */
            public tocRecovered: boolean;

            /** RecoveryReport variablesRecovered. */
            public variablesRecovered: boolean;

            /** RecoveryReport variableCount. */
            public variableCount: (number|Long);

            /** RecoveryReport formsRecovered. */
            public formsRecovered: boolean;

            /** RecoveryReport formDefinitionCount. */
            public formDefinitionCount: (number|Long);

            /** RecoveryReport formRecordCount. */
            public formRecordCount: (number|Long);

            /**
             * Creates a new RecoveryReport instance using the specified properties.
             * @param [properties] Properties to set
             * @returns RecoveryReport instance
             */
            public static create(properties?: revault.bindings.IRecoveryReport): revault.bindings.RecoveryReport;

            /**
             * Encodes the specified RecoveryReport message. Does not implicitly {@link revault.bindings.RecoveryReport.verify|verify} messages.
             * @param message RecoveryReport message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IRecoveryReport, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified RecoveryReport message, length delimited. Does not implicitly {@link revault.bindings.RecoveryReport.verify|verify} messages.
             * @param message RecoveryReport message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IRecoveryReport, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a RecoveryReport message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns RecoveryReport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.RecoveryReport;

            /**
             * Decodes a RecoveryReport message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns RecoveryReport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.RecoveryReport;

            /**
             * Verifies a RecoveryReport message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a RecoveryReport message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns RecoveryReport
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.RecoveryReport;

            /**
             * Creates a plain object from a RecoveryReport message. Also converts values to other types if specified.
             * @param message RecoveryReport
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.RecoveryReport, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this RecoveryReport to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for RecoveryReport
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a KeySlot. */
        interface IKeySlot {

            /** KeySlot id */
            id?: (number|Long|null);

            /** KeySlot protection */
            protection?: (string|null);

            /** KeySlot algorithm */
            algorithm?: (string|null);
        }

        /** Represents a KeySlot. */
        class KeySlot implements IKeySlot {

            /**
             * Constructs a new KeySlot.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IKeySlot);

            /** KeySlot id. */
            public id: (number|Long);

            /** KeySlot protection. */
            public protection: string;

            /** KeySlot algorithm. */
            public algorithm: string;

            /**
             * Creates a new KeySlot instance using the specified properties.
             * @param [properties] Properties to set
             * @returns KeySlot instance
             */
            public static create(properties?: revault.bindings.IKeySlot): revault.bindings.KeySlot;

            /**
             * Encodes the specified KeySlot message. Does not implicitly {@link revault.bindings.KeySlot.verify|verify} messages.
             * @param message KeySlot message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IKeySlot, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified KeySlot message, length delimited. Does not implicitly {@link revault.bindings.KeySlot.verify|verify} messages.
             * @param message KeySlot message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IKeySlot, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a KeySlot message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns KeySlot
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.KeySlot;

            /**
             * Decodes a KeySlot message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns KeySlot
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.KeySlot;

            /**
             * Verifies a KeySlot message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a KeySlot message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns KeySlot
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.KeySlot;

            /**
             * Creates a plain object from a KeySlot message. Also converts values to other types if specified.
             * @param message KeySlot
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.KeySlot, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this KeySlot to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for KeySlot
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a KeySlotList. */
        interface IKeySlotList {

            /** KeySlotList values */
            values?: (revault.bindings.IKeySlot[]|null);
        }

        /** Represents a KeySlotList. */
        class KeySlotList implements IKeySlotList {

            /**
             * Constructs a new KeySlotList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IKeySlotList);

            /** KeySlotList values. */
            public values: revault.bindings.IKeySlot[];

            /**
             * Creates a new KeySlotList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns KeySlotList instance
             */
            public static create(properties?: revault.bindings.IKeySlotList): revault.bindings.KeySlotList;

            /**
             * Encodes the specified KeySlotList message. Does not implicitly {@link revault.bindings.KeySlotList.verify|verify} messages.
             * @param message KeySlotList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IKeySlotList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified KeySlotList message, length delimited. Does not implicitly {@link revault.bindings.KeySlotList.verify|verify} messages.
             * @param message KeySlotList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IKeySlotList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a KeySlotList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns KeySlotList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.KeySlotList;

            /**
             * Decodes a KeySlotList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns KeySlotList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.KeySlotList;

            /**
             * Verifies a KeySlotList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a KeySlotList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns KeySlotList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.KeySlotList;

            /**
             * Creates a plain object from a KeySlotList message. Also converts values to other types if specified.
             * @param message KeySlotList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.KeySlotList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this KeySlotList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for KeySlotList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a CacheStats. */
        interface ICacheStats {

            /** CacheStats limitBytes */
            limitBytes?: (number|Long|null);

            /** CacheStats usedBytes */
            usedBytes?: (number|Long|null);

            /** CacheStats entries */
            entries?: (number|Long|null);

            /** CacheStats hits */
            hits?: (number|Long|null);

            /** CacheStats misses */
            misses?: (number|Long|null);
        }

        /** Represents a CacheStats. */
        class CacheStats implements ICacheStats {

            /**
             * Constructs a new CacheStats.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.ICacheStats);

            /** CacheStats limitBytes. */
            public limitBytes: (number|Long);

            /** CacheStats usedBytes. */
            public usedBytes: (number|Long);

            /** CacheStats entries. */
            public entries: (number|Long);

            /** CacheStats hits. */
            public hits: (number|Long);

            /** CacheStats misses. */
            public misses: (number|Long);

            /**
             * Creates a new CacheStats instance using the specified properties.
             * @param [properties] Properties to set
             * @returns CacheStats instance
             */
            public static create(properties?: revault.bindings.ICacheStats): revault.bindings.CacheStats;

            /**
             * Encodes the specified CacheStats message. Does not implicitly {@link revault.bindings.CacheStats.verify|verify} messages.
             * @param message CacheStats message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.ICacheStats, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified CacheStats message, length delimited. Does not implicitly {@link revault.bindings.CacheStats.verify|verify} messages.
             * @param message CacheStats message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.ICacheStats, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a CacheStats message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns CacheStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.CacheStats;

            /**
             * Decodes a CacheStats message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns CacheStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.CacheStats;

            /**
             * Verifies a CacheStats message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a CacheStats message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns CacheStats
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.CacheStats;

            /**
             * Creates a plain object from a CacheStats message. Also converts values to other types if specified.
             * @param message CacheStats
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.CacheStats, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this CacheStats to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for CacheStats
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an ImportStats. */
        interface IImportStats {

            /** ImportStats hostStatNanos */
            hostStatNanos?: (string|null);

            /** ImportStats hostReadNanos */
            hostReadNanos?: (string|null);

            /** ImportStats framePrepareNanos */
            framePrepareNanos?: (string|null);

            /** ImportStats pageWriteNanos */
            pageWriteNanos?: (string|null);
        }

        /** Represents an ImportStats. */
        class ImportStats implements IImportStats {

            /**
             * Constructs a new ImportStats.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IImportStats);

            /** ImportStats hostStatNanos. */
            public hostStatNanos: string;

            /** ImportStats hostReadNanos. */
            public hostReadNanos: string;

            /** ImportStats framePrepareNanos. */
            public framePrepareNanos: string;

            /** ImportStats pageWriteNanos. */
            public pageWriteNanos: string;

            /**
             * Creates a new ImportStats instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ImportStats instance
             */
            public static create(properties?: revault.bindings.IImportStats): revault.bindings.ImportStats;

            /**
             * Encodes the specified ImportStats message. Does not implicitly {@link revault.bindings.ImportStats.verify|verify} messages.
             * @param message ImportStats message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IImportStats, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ImportStats message, length delimited. Does not implicitly {@link revault.bindings.ImportStats.verify|verify} messages.
             * @param message ImportStats message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IImportStats, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an ImportStats message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ImportStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ImportStats;

            /**
             * Decodes an ImportStats message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ImportStats
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ImportStats;

            /**
             * Verifies an ImportStats message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an ImportStats message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ImportStats
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ImportStats;

            /**
             * Creates a plain object from an ImportStats message. Also converts values to other types if specified.
             * @param message ImportStats
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ImportStats, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ImportStats to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ImportStats
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PageObject. */
        interface IPageObject {

            /** PageObject id */
            id?: (number|Long|null);

            /** PageObject kind */
            kind?: (string|null);

            /** PageObject payloadLen */
            payloadLen?: (number|Long|null);
        }

        /** Represents a PageObject. */
        class PageObject implements IPageObject {

            /**
             * Constructs a new PageObject.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPageObject);

            /** PageObject id. */
            public id: (number|Long);

            /** PageObject kind. */
            public kind: string;

            /** PageObject payloadLen. */
            public payloadLen: (number|Long);

            /**
             * Creates a new PageObject instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PageObject instance
             */
            public static create(properties?: revault.bindings.IPageObject): revault.bindings.PageObject;

            /**
             * Encodes the specified PageObject message. Does not implicitly {@link revault.bindings.PageObject.verify|verify} messages.
             * @param message PageObject message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPageObject, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PageObject message, length delimited. Does not implicitly {@link revault.bindings.PageObject.verify|verify} messages.
             * @param message PageObject message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPageObject, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PageObject message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PageObject
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PageObject;

            /**
             * Decodes a PageObject message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PageObject
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PageObject;

            /**
             * Verifies a PageObject message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PageObject message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PageObject
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PageObject;

            /**
             * Creates a plain object from a PageObject message. Also converts values to other types if specified.
             * @param message PageObject
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PageObject, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PageObject to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PageObject
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PageInspection. */
        interface IPageInspection {

            /** PageInspection offset */
            offset?: (number|Long|null);

            /** PageInspection pageId */
            pageId?: (number|Long|null);

            /** PageInspection sequence */
            sequence?: (number|Long|null);

            /** PageInspection pageSize */
            pageSize?: (number|Long|null);

            /** PageInspection encryptedBodyLen */
            encryptedBodyLen?: (number|Long|null);

            /** PageInspection unusedBytes */
            unusedBytes?: (number|Long|null);

            /** PageInspection objectCount */
            objectCount?: (number|Long|null);

            /** PageInspection objects */
            objects?: (revault.bindings.IPageObject[]|null);
        }

        /** Represents a PageInspection. */
        class PageInspection implements IPageInspection {

            /**
             * Constructs a new PageInspection.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPageInspection);

            /** PageInspection offset. */
            public offset: (number|Long);

            /** PageInspection pageId. */
            public pageId: (number|Long);

            /** PageInspection sequence. */
            public sequence: (number|Long);

            /** PageInspection pageSize. */
            public pageSize: (number|Long);

            /** PageInspection encryptedBodyLen. */
            public encryptedBodyLen: (number|Long);

            /** PageInspection unusedBytes. */
            public unusedBytes: (number|Long);

            /** PageInspection objectCount. */
            public objectCount: (number|Long);

            /** PageInspection objects. */
            public objects: revault.bindings.IPageObject[];

            /**
             * Creates a new PageInspection instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PageInspection instance
             */
            public static create(properties?: revault.bindings.IPageInspection): revault.bindings.PageInspection;

            /**
             * Encodes the specified PageInspection message. Does not implicitly {@link revault.bindings.PageInspection.verify|verify} messages.
             * @param message PageInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPageInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PageInspection message, length delimited. Does not implicitly {@link revault.bindings.PageInspection.verify|verify} messages.
             * @param message PageInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPageInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PageInspection message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PageInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PageInspection;

            /**
             * Decodes a PageInspection message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PageInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PageInspection;

            /**
             * Verifies a PageInspection message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PageInspection message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PageInspection
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PageInspection;

            /**
             * Creates a plain object from a PageInspection message. Also converts values to other types if specified.
             * @param message PageInspection
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PageInspection, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PageInspection to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PageInspection
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PageInspectionList. */
        interface IPageInspectionList {

            /** PageInspectionList values */
            values?: (revault.bindings.IPageInspection[]|null);
        }

        /** Represents a PageInspectionList. */
        class PageInspectionList implements IPageInspectionList {

            /**
             * Constructs a new PageInspectionList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPageInspectionList);

            /** PageInspectionList values. */
            public values: revault.bindings.IPageInspection[];

            /**
             * Creates a new PageInspectionList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PageInspectionList instance
             */
            public static create(properties?: revault.bindings.IPageInspectionList): revault.bindings.PageInspectionList;

            /**
             * Encodes the specified PageInspectionList message. Does not implicitly {@link revault.bindings.PageInspectionList.verify|verify} messages.
             * @param message PageInspectionList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPageInspectionList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PageInspectionList message, length delimited. Does not implicitly {@link revault.bindings.PageInspectionList.verify|verify} messages.
             * @param message PageInspectionList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPageInspectionList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PageInspectionList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PageInspectionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PageInspectionList;

            /**
             * Decodes a PageInspectionList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PageInspectionList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PageInspectionList;

            /**
             * Verifies a PageInspectionList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PageInspectionList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PageInspectionList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PageInspectionList;

            /**
             * Creates a plain object from a PageInspectionList message. Also converts values to other types if specified.
             * @param message PageInspectionList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PageInspectionList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PageInspectionList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PageInspectionList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a FileInspection. */
        interface IFileInspection {

            /** FileInspection lockboxId */
            lockboxId?: (Uint8Array|null);

            /** FileInspection headerReadable */
            headerReadable?: (boolean|null);

            /** FileInspection keyDirectoryGeneration */
            keyDirectoryGeneration?: (number|Long|null);

            /** FileInspection keyDirectoryCopyCount */
            keyDirectoryCopyCount?: (number|Long|null);

            /** FileInspection ownerSigned */
            ownerSigned?: (boolean|null);

            /** FileInspection keySlots */
            keySlots?: (revault.bindings.IKeySlot[]|null);
        }

        /** Represents a FileInspection. */
        class FileInspection implements IFileInspection {

            /**
             * Constructs a new FileInspection.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IFileInspection);

            /** FileInspection lockboxId. */
            public lockboxId: Uint8Array;

            /** FileInspection headerReadable. */
            public headerReadable: boolean;

            /** FileInspection keyDirectoryGeneration. */
            public keyDirectoryGeneration: (number|Long);

            /** FileInspection keyDirectoryCopyCount. */
            public keyDirectoryCopyCount: (number|Long);

            /** FileInspection ownerSigned. */
            public ownerSigned: boolean;

            /** FileInspection keySlots. */
            public keySlots: revault.bindings.IKeySlot[];

            /**
             * Creates a new FileInspection instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FileInspection instance
             */
            public static create(properties?: revault.bindings.IFileInspection): revault.bindings.FileInspection;

            /**
             * Encodes the specified FileInspection message. Does not implicitly {@link revault.bindings.FileInspection.verify|verify} messages.
             * @param message FileInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IFileInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FileInspection message, length delimited. Does not implicitly {@link revault.bindings.FileInspection.verify|verify} messages.
             * @param message FileInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IFileInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FileInspection message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FileInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.FileInspection;

            /**
             * Decodes a FileInspection message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FileInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.FileInspection;

            /**
             * Verifies a FileInspection message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FileInspection message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FileInspection
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.FileInspection;

            /**
             * Creates a plain object from a FileInspection message. Also converts values to other types if specified.
             * @param message FileInspection
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.FileInspection, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FileInspection to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FileInspection
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a ProfileGeneration. */
        interface IProfileGeneration {

            /** ProfileGeneration index */
            index?: (number|null);

            /** ProfileGeneration status */
            status?: (string|null);

            /** ProfileGeneration contactFingerprint */
            contactFingerprint?: (Uint8Array|null);

            /** ProfileGeneration createdAtUnixMs */
            createdAtUnixMs?: (number|Long|null);

            /** ProfileGeneration retiredAtUnixMs */
            retiredAtUnixMs?: (number|Long|null);

            /** ProfileGeneration hasRetiredAt */
            hasRetiredAt?: (boolean|null);
        }

        /** Represents a ProfileGeneration. */
        class ProfileGeneration implements IProfileGeneration {

            /**
             * Constructs a new ProfileGeneration.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IProfileGeneration);

            /** ProfileGeneration index. */
            public index: number;

            /** ProfileGeneration status. */
            public status: string;

            /** ProfileGeneration contactFingerprint. */
            public contactFingerprint: Uint8Array;

            /** ProfileGeneration createdAtUnixMs. */
            public createdAtUnixMs: (number|Long);

            /** ProfileGeneration retiredAtUnixMs. */
            public retiredAtUnixMs: (number|Long);

            /** ProfileGeneration hasRetiredAt. */
            public hasRetiredAt: boolean;

            /**
             * Creates a new ProfileGeneration instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ProfileGeneration instance
             */
            public static create(properties?: revault.bindings.IProfileGeneration): revault.bindings.ProfileGeneration;

            /**
             * Encodes the specified ProfileGeneration message. Does not implicitly {@link revault.bindings.ProfileGeneration.verify|verify} messages.
             * @param message ProfileGeneration message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IProfileGeneration, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ProfileGeneration message, length delimited. Does not implicitly {@link revault.bindings.ProfileGeneration.verify|verify} messages.
             * @param message ProfileGeneration message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IProfileGeneration, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ProfileGeneration message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ProfileGeneration
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ProfileGeneration;

            /**
             * Decodes a ProfileGeneration message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ProfileGeneration
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ProfileGeneration;

            /**
             * Verifies a ProfileGeneration message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ProfileGeneration message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ProfileGeneration
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ProfileGeneration;

            /**
             * Creates a plain object from a ProfileGeneration message. Also converts values to other types if specified.
             * @param message ProfileGeneration
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ProfileGeneration, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ProfileGeneration to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ProfileGeneration
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a ProfileHistory. */
        interface IProfileHistory {

            /** ProfileHistory name */
            name?: (string|null);

            /** ProfileHistory activeGeneration */
            activeGeneration?: (number|null);

            /** ProfileHistory generations */
            generations?: (revault.bindings.IProfileGeneration[]|null);
        }

        /** Represents a ProfileHistory. */
        class ProfileHistory implements IProfileHistory {

            /**
             * Constructs a new ProfileHistory.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IProfileHistory);

            /** ProfileHistory name. */
            public name: string;

            /** ProfileHistory activeGeneration. */
            public activeGeneration: number;

            /** ProfileHistory generations. */
            public generations: revault.bindings.IProfileGeneration[];

            /**
             * Creates a new ProfileHistory instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ProfileHistory instance
             */
            public static create(properties?: revault.bindings.IProfileHistory): revault.bindings.ProfileHistory;

            /**
             * Encodes the specified ProfileHistory message. Does not implicitly {@link revault.bindings.ProfileHistory.verify|verify} messages.
             * @param message ProfileHistory message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IProfileHistory, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ProfileHistory message, length delimited. Does not implicitly {@link revault.bindings.ProfileHistory.verify|verify} messages.
             * @param message ProfileHistory message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IProfileHistory, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ProfileHistory message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ProfileHistory
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ProfileHistory;

            /**
             * Decodes a ProfileHistory message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ProfileHistory
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ProfileHistory;

            /**
             * Verifies a ProfileHistory message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ProfileHistory message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ProfileHistory
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ProfileHistory;

            /**
             * Creates a plain object from a ProfileHistory message. Also converts values to other types if specified.
             * @param message ProfileHistory
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ProfileHistory, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ProfileHistory to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ProfileHistory
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a KnownLockbox. */
        interface IKnownLockbox {

            /** KnownLockbox lockboxId */
            lockboxId?: (Uint8Array|null);

            /** KnownLockbox path */
            path?: (string|null);

            /** KnownLockbox lastSeenUnixMs */
            lastSeenUnixMs?: (number|Long|null);
        }

        /** Represents a KnownLockbox. */
        class KnownLockbox implements IKnownLockbox {

            /**
             * Constructs a new KnownLockbox.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IKnownLockbox);

            /** KnownLockbox lockboxId. */
            public lockboxId: Uint8Array;

            /** KnownLockbox path. */
            public path: string;

            /** KnownLockbox lastSeenUnixMs. */
            public lastSeenUnixMs: (number|Long);

            /**
             * Creates a new KnownLockbox instance using the specified properties.
             * @param [properties] Properties to set
             * @returns KnownLockbox instance
             */
            public static create(properties?: revault.bindings.IKnownLockbox): revault.bindings.KnownLockbox;

            /**
             * Encodes the specified KnownLockbox message. Does not implicitly {@link revault.bindings.KnownLockbox.verify|verify} messages.
             * @param message KnownLockbox message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IKnownLockbox, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified KnownLockbox message, length delimited. Does not implicitly {@link revault.bindings.KnownLockbox.verify|verify} messages.
             * @param message KnownLockbox message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IKnownLockbox, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a KnownLockbox message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns KnownLockbox
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.KnownLockbox;

            /**
             * Decodes a KnownLockbox message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns KnownLockbox
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.KnownLockbox;

            /**
             * Verifies a KnownLockbox message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a KnownLockbox message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns KnownLockbox
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.KnownLockbox;

            /**
             * Creates a plain object from a KnownLockbox message. Also converts values to other types if specified.
             * @param message KnownLockbox
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.KnownLockbox, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this KnownLockbox to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for KnownLockbox
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a KnownLockboxList. */
        interface IKnownLockboxList {

            /** KnownLockboxList values */
            values?: (revault.bindings.IKnownLockbox[]|null);
        }

        /** Represents a KnownLockboxList. */
        class KnownLockboxList implements IKnownLockboxList {

            /**
             * Constructs a new KnownLockboxList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IKnownLockboxList);

            /** KnownLockboxList values. */
            public values: revault.bindings.IKnownLockbox[];

            /**
             * Creates a new KnownLockboxList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns KnownLockboxList instance
             */
            public static create(properties?: revault.bindings.IKnownLockboxList): revault.bindings.KnownLockboxList;

            /**
             * Encodes the specified KnownLockboxList message. Does not implicitly {@link revault.bindings.KnownLockboxList.verify|verify} messages.
             * @param message KnownLockboxList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IKnownLockboxList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified KnownLockboxList message, length delimited. Does not implicitly {@link revault.bindings.KnownLockboxList.verify|verify} messages.
             * @param message KnownLockboxList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IKnownLockboxList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a KnownLockboxList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns KnownLockboxList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.KnownLockboxList;

            /**
             * Decodes a KnownLockboxList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns KnownLockboxList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.KnownLockboxList;

            /**
             * Verifies a KnownLockboxList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a KnownLockboxList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns KnownLockboxList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.KnownLockboxList;

            /**
             * Creates a plain object from a KnownLockboxList message. Also converts values to other types if specified.
             * @param message KnownLockboxList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.KnownLockboxList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this KnownLockboxList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for KnownLockboxList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an AccessSlotLabel. */
        interface IAccessSlotLabel {

            /** AccessSlotLabel lockboxId */
            lockboxId?: (Uint8Array|null);

            /** AccessSlotLabel slotId */
            slotId?: (number|Long|null);

            /** AccessSlotLabel name */
            name?: (string|null);

            /** AccessSlotLabel updatedAtUnixMs */
            updatedAtUnixMs?: (number|Long|null);
        }

        /** Represents an AccessSlotLabel. */
        class AccessSlotLabel implements IAccessSlotLabel {

            /**
             * Constructs a new AccessSlotLabel.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IAccessSlotLabel);

            /** AccessSlotLabel lockboxId. */
            public lockboxId: Uint8Array;

            /** AccessSlotLabel slotId. */
            public slotId: (number|Long);

            /** AccessSlotLabel name. */
            public name: string;

            /** AccessSlotLabel updatedAtUnixMs. */
            public updatedAtUnixMs: (number|Long);

            /**
             * Creates a new AccessSlotLabel instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AccessSlotLabel instance
             */
            public static create(properties?: revault.bindings.IAccessSlotLabel): revault.bindings.AccessSlotLabel;

            /**
             * Encodes the specified AccessSlotLabel message. Does not implicitly {@link revault.bindings.AccessSlotLabel.verify|verify} messages.
             * @param message AccessSlotLabel message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IAccessSlotLabel, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AccessSlotLabel message, length delimited. Does not implicitly {@link revault.bindings.AccessSlotLabel.verify|verify} messages.
             * @param message AccessSlotLabel message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IAccessSlotLabel, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AccessSlotLabel message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns AccessSlotLabel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.AccessSlotLabel;

            /**
             * Decodes an AccessSlotLabel message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns AccessSlotLabel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.AccessSlotLabel;

            /**
             * Verifies an AccessSlotLabel message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AccessSlotLabel message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AccessSlotLabel
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.AccessSlotLabel;

            /**
             * Creates a plain object from an AccessSlotLabel message. Also converts values to other types if specified.
             * @param message AccessSlotLabel
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.AccessSlotLabel, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AccessSlotLabel to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for AccessSlotLabel
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an AccessSlotLabelList. */
        interface IAccessSlotLabelList {

            /** AccessSlotLabelList values */
            values?: (revault.bindings.IAccessSlotLabel[]|null);
        }

        /** Represents an AccessSlotLabelList. */
        class AccessSlotLabelList implements IAccessSlotLabelList {

            /**
             * Constructs a new AccessSlotLabelList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IAccessSlotLabelList);

            /** AccessSlotLabelList values. */
            public values: revault.bindings.IAccessSlotLabel[];

            /**
             * Creates a new AccessSlotLabelList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AccessSlotLabelList instance
             */
            public static create(properties?: revault.bindings.IAccessSlotLabelList): revault.bindings.AccessSlotLabelList;

            /**
             * Encodes the specified AccessSlotLabelList message. Does not implicitly {@link revault.bindings.AccessSlotLabelList.verify|verify} messages.
             * @param message AccessSlotLabelList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IAccessSlotLabelList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AccessSlotLabelList message, length delimited. Does not implicitly {@link revault.bindings.AccessSlotLabelList.verify|verify} messages.
             * @param message AccessSlotLabelList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IAccessSlotLabelList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AccessSlotLabelList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns AccessSlotLabelList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.AccessSlotLabelList;

            /**
             * Decodes an AccessSlotLabelList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns AccessSlotLabelList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.AccessSlotLabelList;

            /**
             * Verifies an AccessSlotLabelList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AccessSlotLabelList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AccessSlotLabelList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.AccessSlotLabelList;

            /**
             * Creates a plain object from an AccessSlotLabelList message. Also converts values to other types if specified.
             * @param message AccessSlotLabelList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.AccessSlotLabelList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AccessSlotLabelList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for AccessSlotLabelList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a StreamChunk. */
        interface IStreamChunk {

            /** StreamChunk path */
            path?: (string|null);

            /** StreamChunk fileOffset */
            fileOffset?: (number|Long|null);

            /** StreamChunk length */
            length?: (number|Long|null);

            /** StreamChunk physicalOffset */
            physicalOffset?: (number|Long|null);

            /** StreamChunk sparse */
            sparse?: (boolean|null);

            /** StreamChunk data */
            data?: (Uint8Array|null);
        }

        /** Represents a StreamChunk. */
        class StreamChunk implements IStreamChunk {

            /**
             * Constructs a new StreamChunk.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IStreamChunk);

            /** StreamChunk path. */
            public path: string;

            /** StreamChunk fileOffset. */
            public fileOffset: (number|Long);

            /** StreamChunk length. */
            public length: (number|Long);

            /** StreamChunk physicalOffset. */
            public physicalOffset: (number|Long);

            /** StreamChunk sparse. */
            public sparse: boolean;

            /** StreamChunk data. */
            public data: Uint8Array;

            /**
             * Creates a new StreamChunk instance using the specified properties.
             * @param [properties] Properties to set
             * @returns StreamChunk instance
             */
            public static create(properties?: revault.bindings.IStreamChunk): revault.bindings.StreamChunk;

            /**
             * Encodes the specified StreamChunk message. Does not implicitly {@link revault.bindings.StreamChunk.verify|verify} messages.
             * @param message StreamChunk message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IStreamChunk, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified StreamChunk message, length delimited. Does not implicitly {@link revault.bindings.StreamChunk.verify|verify} messages.
             * @param message StreamChunk message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IStreamChunk, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a StreamChunk message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns StreamChunk
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.StreamChunk;

            /**
             * Decodes a StreamChunk message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns StreamChunk
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.StreamChunk;

            /**
             * Verifies a StreamChunk message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a StreamChunk message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns StreamChunk
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.StreamChunk;

            /**
             * Creates a plain object from a StreamChunk message. Also converts values to other types if specified.
             * @param message StreamChunk
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.StreamChunk, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this StreamChunk to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for StreamChunk
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a StreamChunkList. */
        interface IStreamChunkList {

            /** StreamChunkList values */
            values?: (revault.bindings.IStreamChunk[]|null);
        }

        /** Represents a StreamChunkList. */
        class StreamChunkList implements IStreamChunkList {

            /**
             * Constructs a new StreamChunkList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IStreamChunkList);

            /** StreamChunkList values. */
            public values: revault.bindings.IStreamChunk[];

            /**
             * Creates a new StreamChunkList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns StreamChunkList instance
             */
            public static create(properties?: revault.bindings.IStreamChunkList): revault.bindings.StreamChunkList;

            /**
             * Encodes the specified StreamChunkList message. Does not implicitly {@link revault.bindings.StreamChunkList.verify|verify} messages.
             * @param message StreamChunkList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IStreamChunkList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified StreamChunkList message, length delimited. Does not implicitly {@link revault.bindings.StreamChunkList.verify|verify} messages.
             * @param message StreamChunkList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IStreamChunkList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a StreamChunkList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns StreamChunkList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.StreamChunkList;

            /**
             * Decodes a StreamChunkList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns StreamChunkList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.StreamChunkList;

            /**
             * Verifies a StreamChunkList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a StreamChunkList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns StreamChunkList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.StreamChunkList;

            /**
             * Creates a plain object from a StreamChunkList message. Also converts values to other types if specified.
             * @param message StreamChunkList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.StreamChunkList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this StreamChunkList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for StreamChunkList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a RuntimeOptions. */
        interface IRuntimeOptions {

            /** RuntimeOptions workloadProfile */
            workloadProfile?: (string|null);

            /** RuntimeOptions workerPolicy */
            workerPolicy?: (string|null);
        }

        /** Represents a RuntimeOptions. */
        class RuntimeOptions implements IRuntimeOptions {

            /**
             * Constructs a new RuntimeOptions.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IRuntimeOptions);

            /** RuntimeOptions workloadProfile. */
            public workloadProfile: string;

            /** RuntimeOptions workerPolicy. */
            public workerPolicy: string;

            /**
             * Creates a new RuntimeOptions instance using the specified properties.
             * @param [properties] Properties to set
             * @returns RuntimeOptions instance
             */
            public static create(properties?: revault.bindings.IRuntimeOptions): revault.bindings.RuntimeOptions;

            /**
             * Encodes the specified RuntimeOptions message. Does not implicitly {@link revault.bindings.RuntimeOptions.verify|verify} messages.
             * @param message RuntimeOptions message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IRuntimeOptions, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified RuntimeOptions message, length delimited. Does not implicitly {@link revault.bindings.RuntimeOptions.verify|verify} messages.
             * @param message RuntimeOptions message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IRuntimeOptions, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a RuntimeOptions message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns RuntimeOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.RuntimeOptions;

            /**
             * Decodes a RuntimeOptions message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns RuntimeOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.RuntimeOptions;

            /**
             * Verifies a RuntimeOptions message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a RuntimeOptions message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns RuntimeOptions
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.RuntimeOptions;

            /**
             * Creates a plain object from a RuntimeOptions message. Also converts values to other types if specified.
             * @param message RuntimeOptions
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.RuntimeOptions, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this RuntimeOptions to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for RuntimeOptions
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a Variable. */
        interface IVariable {

            /** Variable name */
            name?: (string|null);

            /** Variable sensitivity */
            sensitivity?: (string|null);
        }

        /** Represents a Variable. */
        class Variable implements IVariable {

            /**
             * Constructs a new Variable.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IVariable);

            /** Variable name. */
            public name: string;

            /** Variable sensitivity. */
            public sensitivity: string;

            /**
             * Creates a new Variable instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Variable instance
             */
            public static create(properties?: revault.bindings.IVariable): revault.bindings.Variable;

            /**
             * Encodes the specified Variable message. Does not implicitly {@link revault.bindings.Variable.verify|verify} messages.
             * @param message Variable message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IVariable, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Variable message, length delimited. Does not implicitly {@link revault.bindings.Variable.verify|verify} messages.
             * @param message Variable message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IVariable, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Variable message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Variable
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.Variable;

            /**
             * Decodes a Variable message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Variable
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.Variable;

            /**
             * Verifies a Variable message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Variable message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Variable
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.Variable;

            /**
             * Creates a plain object from a Variable message. Also converts values to other types if specified.
             * @param message Variable
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.Variable, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Variable to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Variable
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a VariableList. */
        interface IVariableList {

            /** VariableList values */
            values?: (revault.bindings.IVariable[]|null);
        }

        /** Represents a VariableList. */
        class VariableList implements IVariableList {

            /**
             * Constructs a new VariableList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IVariableList);

            /** VariableList values. */
            public values: revault.bindings.IVariable[];

            /**
             * Creates a new VariableList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns VariableList instance
             */
            public static create(properties?: revault.bindings.IVariableList): revault.bindings.VariableList;

            /**
             * Encodes the specified VariableList message. Does not implicitly {@link revault.bindings.VariableList.verify|verify} messages.
             * @param message VariableList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IVariableList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified VariableList message, length delimited. Does not implicitly {@link revault.bindings.VariableList.verify|verify} messages.
             * @param message VariableList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IVariableList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a VariableList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns VariableList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.VariableList;

            /**
             * Decodes a VariableList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns VariableList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.VariableList;

            /**
             * Verifies a VariableList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a VariableList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns VariableList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.VariableList;

            /**
             * Creates a plain object from a VariableList message. Also converts values to other types if specified.
             * @param message VariableList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.VariableList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this VariableList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for VariableList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an OptionalString. */
        interface IOptionalString {

            /** OptionalString present */
            present?: (boolean|null);

            /** OptionalString value */
            value?: (string|null);
        }

        /** Represents an OptionalString. */
        class OptionalString implements IOptionalString {

            /**
             * Constructs a new OptionalString.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IOptionalString);

            /** OptionalString present. */
            public present: boolean;

            /** OptionalString value. */
            public value: string;

            /**
             * Creates a new OptionalString instance using the specified properties.
             * @param [properties] Properties to set
             * @returns OptionalString instance
             */
            public static create(properties?: revault.bindings.IOptionalString): revault.bindings.OptionalString;

            /**
             * Encodes the specified OptionalString message. Does not implicitly {@link revault.bindings.OptionalString.verify|verify} messages.
             * @param message OptionalString message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IOptionalString, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified OptionalString message, length delimited. Does not implicitly {@link revault.bindings.OptionalString.verify|verify} messages.
             * @param message OptionalString message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IOptionalString, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an OptionalString message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns OptionalString
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.OptionalString;

            /**
             * Decodes an OptionalString message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns OptionalString
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.OptionalString;

            /**
             * Verifies an OptionalString message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an OptionalString message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns OptionalString
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.OptionalString;

            /**
             * Creates a plain object from an OptionalString message. Also converts values to other types if specified.
             * @param message OptionalString
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.OptionalString, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this OptionalString to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for OptionalString
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an OwnerInspection. */
        interface IOwnerInspection {

            /** OwnerInspection signed */
            signed?: (boolean|null);

            /** OwnerInspection fingerprint */
            fingerprint?: (string|null);

            /** OwnerInspection hasFingerprint */
            hasFingerprint?: (boolean|null);
        }

        /** Represents an OwnerInspection. */
        class OwnerInspection implements IOwnerInspection {

            /**
             * Constructs a new OwnerInspection.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IOwnerInspection);

            /** OwnerInspection signed. */
            public signed: boolean;

            /** OwnerInspection fingerprint. */
            public fingerprint: string;

            /** OwnerInspection hasFingerprint. */
            public hasFingerprint: boolean;

            /**
             * Creates a new OwnerInspection instance using the specified properties.
             * @param [properties] Properties to set
             * @returns OwnerInspection instance
             */
            public static create(properties?: revault.bindings.IOwnerInspection): revault.bindings.OwnerInspection;

            /**
             * Encodes the specified OwnerInspection message. Does not implicitly {@link revault.bindings.OwnerInspection.verify|verify} messages.
             * @param message OwnerInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IOwnerInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified OwnerInspection message, length delimited. Does not implicitly {@link revault.bindings.OwnerInspection.verify|verify} messages.
             * @param message OwnerInspection message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IOwnerInspection, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an OwnerInspection message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns OwnerInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.OwnerInspection;

            /**
             * Decodes an OwnerInspection message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns OwnerInspection
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.OwnerInspection;

            /**
             * Verifies an OwnerInspection message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an OwnerInspection message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns OwnerInspection
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.OwnerInspection;

            /**
             * Creates a plain object from an OwnerInspection message. Also converts values to other types if specified.
             * @param message OwnerInspection
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.OwnerInspection, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this OwnerInspection to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for OwnerInspection
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a Contact. */
        interface IContact {

            /** Contact name */
            name?: (string|null);

            /** Contact key */
            key?: (Uint8Array|null);
        }

        /** Represents a Contact. */
        class Contact implements IContact {

            /**
             * Constructs a new Contact.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IContact);

            /** Contact name. */
            public name: string;

            /** Contact key. */
            public key: Uint8Array;

            /**
             * Creates a new Contact instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Contact instance
             */
            public static create(properties?: revault.bindings.IContact): revault.bindings.Contact;

            /**
             * Encodes the specified Contact message. Does not implicitly {@link revault.bindings.Contact.verify|verify} messages.
             * @param message Contact message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IContact, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Contact message, length delimited. Does not implicitly {@link revault.bindings.Contact.verify|verify} messages.
             * @param message Contact message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IContact, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Contact message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Contact
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.Contact;

            /**
             * Decodes a Contact message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Contact
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.Contact;

            /**
             * Verifies a Contact message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Contact message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Contact
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.Contact;

            /**
             * Creates a plain object from a Contact message. Also converts values to other types if specified.
             * @param message Contact
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.Contact, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Contact to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Contact
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a ContactList. */
        interface IContactList {

            /** ContactList values */
            values?: (revault.bindings.IContact[]|null);
        }

        /** Represents a ContactList. */
        class ContactList implements IContactList {

            /**
             * Constructs a new ContactList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IContactList);

            /** ContactList values. */
            public values: revault.bindings.IContact[];

            /**
             * Creates a new ContactList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ContactList instance
             */
            public static create(properties?: revault.bindings.IContactList): revault.bindings.ContactList;

            /**
             * Encodes the specified ContactList message. Does not implicitly {@link revault.bindings.ContactList.verify|verify} messages.
             * @param message ContactList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IContactList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ContactList message, length delimited. Does not implicitly {@link revault.bindings.ContactList.verify|verify} messages.
             * @param message ContactList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IContactList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ContactList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ContactList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ContactList;

            /**
             * Decodes a ContactList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ContactList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ContactList;

            /**
             * Verifies a ContactList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ContactList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ContactList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ContactList;

            /**
             * Creates a plain object from a ContactList message. Also converts values to other types if specified.
             * @param message ContactList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ContactList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ContactList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ContactList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a ProfileHistoryList. */
        interface IProfileHistoryList {

            /** ProfileHistoryList values */
            values?: (revault.bindings.IProfileHistory[]|null);
        }

        /** Represents a ProfileHistoryList. */
        class ProfileHistoryList implements IProfileHistoryList {

            /**
             * Constructs a new ProfileHistoryList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IProfileHistoryList);

            /** ProfileHistoryList values. */
            public values: revault.bindings.IProfileHistory[];

            /**
             * Creates a new ProfileHistoryList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ProfileHistoryList instance
             */
            public static create(properties?: revault.bindings.IProfileHistoryList): revault.bindings.ProfileHistoryList;

            /**
             * Encodes the specified ProfileHistoryList message. Does not implicitly {@link revault.bindings.ProfileHistoryList.verify|verify} messages.
             * @param message ProfileHistoryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IProfileHistoryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ProfileHistoryList message, length delimited. Does not implicitly {@link revault.bindings.ProfileHistoryList.verify|verify} messages.
             * @param message ProfileHistoryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IProfileHistoryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ProfileHistoryList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ProfileHistoryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ProfileHistoryList;

            /**
             * Decodes a ProfileHistoryList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ProfileHistoryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ProfileHistoryList;

            /**
             * Verifies a ProfileHistoryList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ProfileHistoryList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ProfileHistoryList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ProfileHistoryList;

            /**
             * Creates a plain object from a ProfileHistoryList message. Also converts values to other types if specified.
             * @param message ProfileHistoryList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ProfileHistoryList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ProfileHistoryList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ProfileHistoryList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an AgentEntry. */
        interface IAgentEntry {

            /** AgentEntry id */
            id?: (string|null);

            /** AgentEntry path */
            path?: (string|null);
        }

        /** Represents an AgentEntry. */
        class AgentEntry implements IAgentEntry {

            /**
             * Constructs a new AgentEntry.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IAgentEntry);

            /** AgentEntry id. */
            public id: string;

            /** AgentEntry path. */
            public path: string;

            /**
             * Creates a new AgentEntry instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AgentEntry instance
             */
            public static create(properties?: revault.bindings.IAgentEntry): revault.bindings.AgentEntry;

            /**
             * Encodes the specified AgentEntry message. Does not implicitly {@link revault.bindings.AgentEntry.verify|verify} messages.
             * @param message AgentEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IAgentEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AgentEntry message, length delimited. Does not implicitly {@link revault.bindings.AgentEntry.verify|verify} messages.
             * @param message AgentEntry message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IAgentEntry, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AgentEntry message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns AgentEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.AgentEntry;

            /**
             * Decodes an AgentEntry message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns AgentEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.AgentEntry;

            /**
             * Verifies an AgentEntry message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AgentEntry message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AgentEntry
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.AgentEntry;

            /**
             * Creates a plain object from an AgentEntry message. Also converts values to other types if specified.
             * @param message AgentEntry
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.AgentEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AgentEntry to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for AgentEntry
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an AgentEntryList. */
        interface IAgentEntryList {

            /** AgentEntryList values */
            values?: (revault.bindings.IAgentEntry[]|null);
        }

        /** Represents an AgentEntryList. */
        class AgentEntryList implements IAgentEntryList {

            /**
             * Constructs a new AgentEntryList.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IAgentEntryList);

            /** AgentEntryList values. */
            public values: revault.bindings.IAgentEntry[];

            /**
             * Creates a new AgentEntryList instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AgentEntryList instance
             */
            public static create(properties?: revault.bindings.IAgentEntryList): revault.bindings.AgentEntryList;

            /**
             * Encodes the specified AgentEntryList message. Does not implicitly {@link revault.bindings.AgentEntryList.verify|verify} messages.
             * @param message AgentEntryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IAgentEntryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AgentEntryList message, length delimited. Does not implicitly {@link revault.bindings.AgentEntryList.verify|verify} messages.
             * @param message AgentEntryList message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IAgentEntryList, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AgentEntryList message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns AgentEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.AgentEntryList;

            /**
             * Decodes an AgentEntryList message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns AgentEntryList
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.AgentEntryList;

            /**
             * Verifies an AgentEntryList message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AgentEntryList message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AgentEntryList
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.AgentEntryList;

            /**
             * Creates a plain object from an AgentEntryList message. Also converts values to other types if specified.
             * @param message AgentEntryList
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.AgentEntryList, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AgentEntryList to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for AgentEntryList
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a SleepSupport. */
        interface ISleepSupport {

            /** SleepSupport suspendNotifications */
            suspendNotifications?: (boolean|null);

            /** SleepSupport sleepInhibition */
            sleepInhibition?: (boolean|null);

            /** SleepSupport supported */
            supported?: (boolean|null);
        }

        /** Represents a SleepSupport. */
        class SleepSupport implements ISleepSupport {

            /**
             * Constructs a new SleepSupport.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.ISleepSupport);

            /** SleepSupport suspendNotifications. */
            public suspendNotifications: boolean;

            /** SleepSupport sleepInhibition. */
            public sleepInhibition: boolean;

            /** SleepSupport supported. */
            public supported: boolean;

            /**
             * Creates a new SleepSupport instance using the specified properties.
             * @param [properties] Properties to set
             * @returns SleepSupport instance
             */
            public static create(properties?: revault.bindings.ISleepSupport): revault.bindings.SleepSupport;

            /**
             * Encodes the specified SleepSupport message. Does not implicitly {@link revault.bindings.SleepSupport.verify|verify} messages.
             * @param message SleepSupport message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.ISleepSupport, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified SleepSupport message, length delimited. Does not implicitly {@link revault.bindings.SleepSupport.verify|verify} messages.
             * @param message SleepSupport message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.ISleepSupport, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a SleepSupport message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns SleepSupport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.SleepSupport;

            /**
             * Decodes a SleepSupport message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns SleepSupport
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.SleepSupport;

            /**
             * Verifies a SleepSupport message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a SleepSupport message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns SleepSupport
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.SleepSupport;

            /**
             * Creates a plain object from a SleepSupport message. Also converts values to other types if specified.
             * @param message SleepSupport
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.SleepSupport, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this SleepSupport to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for SleepSupport
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a PlatformStatus. */
        interface IPlatformStatus {

            /** PlatformStatus supported */
            supported?: (boolean|null);

            /** PlatformStatus disabled */
            disabled?: (boolean|null);

            /** PlatformStatus scope */
            scope?: (string|null);

            /** PlatformStatus backend */
            backend?: (string|null);

            /** PlatformStatus item */
            item?: (string|null);
        }

        /** Represents a PlatformStatus. */
        class PlatformStatus implements IPlatformStatus {

            /**
             * Constructs a new PlatformStatus.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IPlatformStatus);

            /** PlatformStatus supported. */
            public supported: boolean;

            /** PlatformStatus disabled. */
            public disabled: boolean;

            /** PlatformStatus scope. */
            public scope: string;

            /** PlatformStatus backend. */
            public backend: string;

            /** PlatformStatus item. */
            public item: string;

            /**
             * Creates a new PlatformStatus instance using the specified properties.
             * @param [properties] Properties to set
             * @returns PlatformStatus instance
             */
            public static create(properties?: revault.bindings.IPlatformStatus): revault.bindings.PlatformStatus;

            /**
             * Encodes the specified PlatformStatus message. Does not implicitly {@link revault.bindings.PlatformStatus.verify|verify} messages.
             * @param message PlatformStatus message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IPlatformStatus, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified PlatformStatus message, length delimited. Does not implicitly {@link revault.bindings.PlatformStatus.verify|verify} messages.
             * @param message PlatformStatus message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IPlatformStatus, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a PlatformStatus message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns PlatformStatus
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.PlatformStatus;

            /**
             * Decodes a PlatformStatus message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns PlatformStatus
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.PlatformStatus;

            /**
             * Verifies a PlatformStatus message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a PlatformStatus message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns PlatformStatus
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.PlatformStatus;

            /**
             * Creates a plain object from a PlatformStatus message. Also converts values to other types if specified.
             * @param message PlatformStatus
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.PlatformStatus, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this PlatformStatus to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for PlatformStatus
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a StringValue. */
        interface IStringValue {

            /** StringValue value */
            value?: (string|null);
        }

        /** Represents a StringValue. */
        class StringValue implements IStringValue {

            /**
             * Constructs a new StringValue.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IStringValue);

            /** StringValue value. */
            public value: string;

            /**
             * Creates a new StringValue instance using the specified properties.
             * @param [properties] Properties to set
             * @returns StringValue instance
             */
            public static create(properties?: revault.bindings.IStringValue): revault.bindings.StringValue;

            /**
             * Encodes the specified StringValue message. Does not implicitly {@link revault.bindings.StringValue.verify|verify} messages.
             * @param message StringValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IStringValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified StringValue message, length delimited. Does not implicitly {@link revault.bindings.StringValue.verify|verify} messages.
             * @param message StringValue message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IStringValue, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a StringValue message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns StringValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.StringValue;

            /**
             * Decodes a StringValue message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns StringValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.StringValue;

            /**
             * Verifies a StringValue message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a StringValue message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns StringValue
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.StringValue;

            /**
             * Creates a plain object from a StringValue message. Also converts values to other types if specified.
             * @param message StringValue
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.StringValue, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this StringValue to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for StringValue
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a VaultBackupManifest. */
        interface IVaultBackupManifest {

            /** VaultBackupManifest formatVersion */
            formatVersion?: (number|null);

            /** VaultBackupManifest createdAtUnixMs */
            createdAtUnixMs?: (number|Long|null);

            /** VaultBackupManifest vaultFileName */
            vaultFileName?: (string|null);

            /** VaultBackupManifest vaultSize */
            vaultSize?: (number|Long|null);

            /** VaultBackupManifest vaultSha256 */
            vaultSha256?: (string|null);
        }

        /** Represents a VaultBackupManifest. */
        class VaultBackupManifest implements IVaultBackupManifest {

            /**
             * Constructs a new VaultBackupManifest.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IVaultBackupManifest);

            /** VaultBackupManifest formatVersion. */
            public formatVersion: number;

            /** VaultBackupManifest createdAtUnixMs. */
            public createdAtUnixMs: (number|Long);

            /** VaultBackupManifest vaultFileName. */
            public vaultFileName: string;

            /** VaultBackupManifest vaultSize. */
            public vaultSize: (number|Long);

            /** VaultBackupManifest vaultSha256. */
            public vaultSha256: string;

            /**
             * Creates a new VaultBackupManifest instance using the specified properties.
             * @param [properties] Properties to set
             * @returns VaultBackupManifest instance
             */
            public static create(properties?: revault.bindings.IVaultBackupManifest): revault.bindings.VaultBackupManifest;

            /**
             * Encodes the specified VaultBackupManifest message. Does not implicitly {@link revault.bindings.VaultBackupManifest.verify|verify} messages.
             * @param message VaultBackupManifest message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IVaultBackupManifest, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified VaultBackupManifest message, length delimited. Does not implicitly {@link revault.bindings.VaultBackupManifest.verify|verify} messages.
             * @param message VaultBackupManifest message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IVaultBackupManifest, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a VaultBackupManifest message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns VaultBackupManifest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.VaultBackupManifest;

            /**
             * Decodes a VaultBackupManifest message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns VaultBackupManifest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.VaultBackupManifest;

            /**
             * Verifies a VaultBackupManifest message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a VaultBackupManifest message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns VaultBackupManifest
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.VaultBackupManifest;

            /**
             * Creates a plain object from a VaultBackupManifest message. Also converts values to other types if specified.
             * @param message VaultBackupManifest
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.VaultBackupManifest, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this VaultBackupManifest to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for VaultBackupManifest
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of an ErrorDetails. */
        interface IErrorDetails {

            /** ErrorDetails category */
            category?: (string|null);

            /** ErrorDetails artifactKind */
            artifactKind?: (string|null);

            /** ErrorDetails foundVersion */
            foundVersion?: (number|null);

            /** ErrorDetails supportedVersion */
            supportedVersion?: (number|null);

            /** ErrorDetails message */
            message?: (string|null);

            /** ErrorDetails guidance */
            guidance?: (string|null);
        }

        /** Represents an ErrorDetails. */
        class ErrorDetails implements IErrorDetails {

            /**
             * Constructs a new ErrorDetails.
             * @param [properties] Properties to set
             */
            constructor(properties?: revault.bindings.IErrorDetails);

            /** ErrorDetails category. */
            public category: string;

            /** ErrorDetails artifactKind. */
            public artifactKind: string;

            /** ErrorDetails foundVersion. */
            public foundVersion: number;

            /** ErrorDetails supportedVersion. */
            public supportedVersion: number;

            /** ErrorDetails message. */
            public message: string;

            /** ErrorDetails guidance. */
            public guidance: string;

            /**
             * Creates a new ErrorDetails instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ErrorDetails instance
             */
            public static create(properties?: revault.bindings.IErrorDetails): revault.bindings.ErrorDetails;

            /**
             * Encodes the specified ErrorDetails message. Does not implicitly {@link revault.bindings.ErrorDetails.verify|verify} messages.
             * @param message ErrorDetails message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: revault.bindings.IErrorDetails, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ErrorDetails message, length delimited. Does not implicitly {@link revault.bindings.ErrorDetails.verify|verify} messages.
             * @param message ErrorDetails message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: revault.bindings.IErrorDetails, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an ErrorDetails message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ErrorDetails
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): revault.bindings.ErrorDetails;

            /**
             * Decodes an ErrorDetails message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ErrorDetails
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): revault.bindings.ErrorDetails;

            /**
             * Verifies an ErrorDetails message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an ErrorDetails message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ErrorDetails
             */
            public static fromObject(object: { [k: string]: any }): revault.bindings.ErrorDetails;

            /**
             * Creates a plain object from an ErrorDetails message. Also converts values to other types if specified.
             * @param message ErrorDetails
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: revault.bindings.ErrorDetails, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ErrorDetails to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ErrorDetails
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }
}
