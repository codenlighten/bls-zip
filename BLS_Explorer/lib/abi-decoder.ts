import type { ContractCall, AssetTransfer } from './types';

export function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }
  return bytes;
}

export function bytesToString(bytes: Uint8Array): string {
  return new TextDecoder('utf-8').decode(bytes);
}

export function decodeAssetTransfer(hexData: string): AssetTransfer | null {
  try {
    const bytes = hexToBytes(hexData);
    const jsonString = bytesToString(bytes);
    const data = JSON.parse(jsonString);

    if (data.type === 'asset_transfer') {
      return {
        type: 'asset_transfer',
        from_address: data.from_address,
        to_address: data.to_address,
        asset_id: data.asset_id,
        quantity: data.quantity,
        price: data.price,
        metadata: data.metadata,
      };
    }

    return null;
  } catch (error) {
    return null;
  }
}

export function decodeContractCall(hexData: string): ContractCall | null {
  try {
    const bytes = hexToBytes(hexData);

    const nameLen = (bytes[0] << 8) | bytes[1];

    if (nameLen > bytes.length - 2) {
      return null;
    }

    const functionNameBytes = bytes.slice(2, 2 + nameLen);
    const functionName = bytesToString(functionNameBytes);

    const argsBytes = bytes.slice(2 + nameLen);

    let args: Record<string, any> = {};
    if (argsBytes.length > 0) {
      try {
        const argsString = bytesToString(argsBytes);
        args = JSON.parse(argsString);
      } catch {
        args = { raw: Array.from(argsBytes) };
      }
    }

    return {
      function_name: functionName,
      args,
    };
  } catch (error) {
    return null;
  }
}

export function detectDataType(hexData: string): 'asset_transfer' | 'contract_call' | 'unknown' {
  const assetTransfer = decodeAssetTransfer(hexData);
  if (assetTransfer) return 'asset_transfer';

  const contractCall = decodeContractCall(hexData);
  if (contractCall) return 'contract_call';

  return 'unknown';
}

export function encodeContractCall(functionName: string, args: Record<string, any>): string {
  const encoder = new TextEncoder();
  const nameBytes = encoder.encode(functionName);
  const nameLen = nameBytes.length;

  const argsString = JSON.stringify(args);
  const argsBytes = encoder.encode(argsString);

  const result = new Uint8Array(2 + nameBytes.length + argsBytes.length);
  result[0] = (nameLen >> 8) & 0xFF;
  result[1] = nameLen & 0xFF;
  result.set(nameBytes, 2);
  result.set(argsBytes, 2 + nameBytes.length);

  return '0x' + Array.from(result).map(b => b.toString(16).padStart(2, '0')).join('');
}

export function encodeAssetTransfer(transfer: AssetTransfer): string {
  const jsonString = JSON.stringify(transfer);
  const encoder = new TextEncoder();
  const bytes = encoder.encode(jsonString);
  return '0x' + Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}
