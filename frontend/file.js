const button = document.querySelector('button');
const loader = document.querySelector('.loader');
const availability = document.querySelector('.availability');
const errorP = document.querySelector('.error');

const uuid = document.querySelector('input#uuid').value;
const availableTill = document.querySelector('input#available_till').value;
const iv = document.querySelector('input#iv').value;
const key = document.querySelector('input#key').value;
const mimeType = document.querySelector('input#mime_type').value;
const fileName = document.querySelector('input#file_name').value;

var file = null;
var state = 0;
var currentProgress = 0;
var intervalId;


const startProgress = () => {
    intervalId = setInterval(() => {
        if (currentProgress < 80) {
            currentProgress += 5;
        } else if (currentProgress < 95 && state === 1) {
            currentProgress += 5;
        } else if (state === 2) {
            currentProgress = 100;
            clearInterval(intervalId);
        }
        progress(currentProgress);
    }, 350);
}

const stopProgress = () => {
    clearInterval(intervalId);
    setTimeout(() => {
        state = 0;
        currentProgress = 0;
    }, 350);
    loader.classList.add('hidden');
}

button.addEventListener('click', () => {
    if (file !== null) {
        return handleFile(null);
    }

    startProgress();

    const xhr = new XMLHttpRequest();
    xhr.open('GET', `/api/file/${uuid}/download`, true);
    xhr.responseType = 'arraybuffer';

    xhr.onload = () => {
        if (xhr.status === 200) {
            handleFile(xhr.response);
        } else {
            const data = JSON.parse(xhr.responseText);
            error(data.message);
        }
    }

    xhr.onerror = () => {
        error('An error occured while downloading file.');
    }

    xhr.send();
});

const progress = percent => {
    loader.classList.remove('hidden');
    loader.querySelector('div.inner').style.width = `${percent}%`;
}

const success = () => {
    state = 2;
}

const error = msg => {
    stopProgress();
    errorP.classList.remove('hidden');
    errorP.innerText = msg;
}


const handleFile = async bytes => {
    state = 1;
    currentProgress = 80;

    if (file === null) {
        file = await decryptFile(
            bytes,
            key,
            iv,
            mimeType,
            fileName
        );
    }

    const url = URL.createObjectURL(file);

    const a = document.createElement('a');
    a.href = url;
    a.download = file.name;
    a.style.display = 'none';
    document.body.appendChild(a);

    a.onclick = () => {
        setTimeout(() => {
            URL.revokeObjectURL(url)
            document.body.removeChild(a);
        }, 500);
    };

    success();

    setTimeout(() => {
        a.click();
    }, 350);
}

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

const decryptFile = async (encryptedData, encodedKey, encodedIv, mimeType, fileName) => {
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
        encryptedData
    );

    return new File(
        [
            decryptedData
        ],
        fileName,
        {
            type: mimeType
        }
    );
}
