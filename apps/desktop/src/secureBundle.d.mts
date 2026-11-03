export function sealBundle(payload: unknown, passphrase: string): Promise<string>;
export function openBundle(encoded: string, passphrase: string): Promise<{schema:string;source_device_id:string;source_device_name:string;records:unknown[];truncated?:boolean}>;
