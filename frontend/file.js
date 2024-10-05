const base64UrlToArrayBuffer = string => {
    const base64String = string.replace(/-/g, '+').replace(/_/g, '/');
    const binaryString = atob(base64String);
    const len = binaryString.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
}

const decryptFile = async (encryptedData, encodedKey, encodedIv, mimeType, fileType) => {
    const rawKey = base64UrlToArrayBuffer(encodedKey);
    const iv = base64UrlToArrayBuffer(encodedIv);

    const key = await crypto.subtle.importKey(
        'raw',
        rawKey,
        {
            name: 'AES-GCM',
            length: 256
        },
        false,
        [
            "decrypt"
        ]
    );

    const decryptedData = await crypto.subtle.decrypt(
        {
            name: 'AES-GCM',
            iv
        },
        key,
        encryptedData // ArrayBuffer
    );

    return new File(
        [
            decryptedData
        ],
        `file.${fileType}`,
        {
            type: mimeType
        }
    );
}