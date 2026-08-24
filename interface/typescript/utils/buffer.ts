export class Buffer {
	public len = 0;

	private buf: Uint8Array;
	private view: DataView;
	private pos = 0;

	constructor(initialBuf: Uint8Array = new Uint8Array()) {
		this.buf = initialBuf;
		this.view = new DataView(this.buf.buffer);
		this.len = initialBuf.byteLength;
	}

	private cup(n: number): void {
		const needed = this.len + n;
		if (needed <= this.buf.byteLength) return;

		let newCap = Math.max(this.buf.byteLength * 2, needed);
		const newBuf = new Uint8Array(newCap);
		newBuf.set(this.buf.subarray(0, this.len));

		this.buf = newBuf;
		this.view = new DataView(this.buf.buffer);
	}

	public cursorPos(): number {
		return this.pos;
	}

	public setCursorPos(pos: number): void {
		this.pos = pos;
	}

	public remaining(): number {
		return this.len - this.pos;
	}

	public toUint8Array(): Uint8Array {
		return this.buf.subarray(0, this.len);
	}

	public writeU8(value: number): void {
		this.cup(1);
		this.view.setUint8(this.len, value);
		this.len += 1;
	}

	public writeU16(value: number): void {
		this.cup(2);
		this.view.setUint16(this.len, value);
		this.len += 2;
	}

	public writeU32(value: number): void {
		this.cup(4);
		this.view.setUint32(this.len, value);
		this.len += 4;
	}

	public writeU64(value: bigint): void {
		this.cup(8);
		this.view.setBigUint64(this.len, value);
		this.len += 8;
	}

	public writeI32(value: number): void {
		this.cup(4);
		this.view.setInt32(this.len, value);
		this.len += 4;
	}

	public writeI64(value: bigint): void {
		this.cup(8);
		this.view.setBigInt64(this.len, value);
		this.len += 8;
	}

	public writeF32(value: number): void {
		this.cup(4);
		this.view.setFloat32(this.len, value);
		this.len += 4;
	}

	public writeF64(value: number): void {
		this.cup(8);
		this.view.setFloat64(this.len, value);
		this.len += 8;
	}

	public writeBytes(value: Uint8Array): void {
		this.cup(value.byteLength);
		this.buf.set(value, this.len);
		this.len += value.byteLength;
	}

	public writeBoolean(value: boolean): void {
		value ? this.writeU8(0x01) : this.writeU8(0x00);
	}

	public writeString(value: string): void {
		const encoder = new TextEncoder();
		const bytes = encoder.encode(value);
		this.writeU16(bytes.length);
		this.writeBytes(bytes);
	}

	public writeOption<T>(value: T | null | undefined, write: (some: T) => void): void {
		if (value === null || typeof value === "undefined") {
			this.writeU8(0x00);
		} else {
			this.writeU8(0x01);
			write(value);
		}
	}

	public readU8(): number {
		const v = this.view.getUint8(this.pos);
		this.pos += 1;
		return v;
	}

	public readU16(): number {
		const v = this.view.getUint16(this.pos);
		this.pos += 2;
		return v;
	}

	public readU32(): number {
		const v = this.view.getUint32(this.pos);
		this.pos += 4;
		return v;
	}

	public readU64(): bigint {
		const v = this.view.getBigUint64(this.pos);
		this.pos += 8;
		return v;
	}

	public readI32(): number {
		const v = this.view.getInt32(this.pos);
		this.pos += 4;
		return v;
	}

	public readI64(): bigint {
		const v = this.view.getBigInt64(this.pos);
		this.pos += 8;
		return v;
	}

	public readF32(): number {
		const v = this.view.getFloat32(this.pos);
		this.pos += 4;
		return v;
	}

	public readF64(): number {
		const v = this.view.getFloat64(this.pos);
		this.pos += 8;
		return v;
	}

	public readBoolean(): boolean {
		return this.readU8() === 0x01;
	}

	public readBytes(n: number): Uint8Array {
		const start = this.pos;
		this.pos += n;
		return this.buf.subarray(start, start + n);
	}

	public readString(): string {
		const len = this.readU16();
		const bytes = this.readBytes(len);
		const decoder = new TextDecoder("utf-8");
		return decoder.decode(bytes);
	}

	public readOption<T>(read: () => T): T | null {
		const some = this.readU8();
		if (some === 0x00) return null;
		return read();
	}
}
