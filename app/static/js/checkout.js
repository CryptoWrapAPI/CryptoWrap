const API_ENDPOINT = '/api/v1/deposit/check';
const POLL_INTERVAL = 5000;

let pollIntervalId = null;
let paymentData = null;
let redirectUrl = null;

function getUuidFromUrl() {
    const params = new URLSearchParams(window.location.search);
    return params.get('uuid') || params.get('deposit_uuid');
}

function formatStatusText(status) {
    switch (status) {
        case 'waiting':
            return 'Waiting for payment...';
        case 'detected':
            return 'Payment detected!';
        case 'confirmed':
            return 'Payment confirmed!';
        case 'expired':
            return 'Payment expired';
        default:
            return 'Unknown status';
    }
}

function updateUI(data) {
    paymentData = data;
    
    redirectUrl = data.redirect_url || null;
    
    document.getElementById('wallet-address').value = data.wallet_address;
    document.getElementById('deposit-id').textContent = data.deposit_uuid;
    
    const qrUrl = `/api/v1/qr?size=200x200&data=${encodeURIComponent(data.wallet_address)}`;
    document.getElementById('qr-image').src = qrUrl;
    document.getElementById('qr-modal-image').src = qrUrl.replace('size=200x200', 'size=400x400');
    
    const statusIndicator = document.getElementById('status-indicator');
    const statusText = document.getElementById('status-text');
    const statusDot = statusIndicator.querySelector('.status-dot');
    const amountReceived = document.getElementById('amount-received');

    amountReceived.textContent = data.amount_received;

    statusDot.classList.remove('waiting', 'detected', 'confirmed', 'expired');
    statusDot.classList.add(data.payment_status);
    statusText.textContent = formatStatusText(data.payment_status);

    const checkBtn = document.getElementById('check-btn');
    if (data.payment_status === 'waiting') {
        checkBtn.style.display = 'flex';
    } else {
        checkBtn.style.display = 'none';
    }

    const receivedAmount = parseFloat(data.amount_received);
    if (receivedAmount > 0) {
        showSuccess();
    }

    if (data.payment_status === 'confirmed') {
        stopPolling();
    }
}

function showSuccess() {
    const checkBtn = document.getElementById('check-btn');
    const successSection = document.getElementById('success-section');
    const returnBtn = document.getElementById('return-btn');

    checkBtn.style.display = 'none';
    successSection.style.display = 'block';

    if (redirectUrl) {
        returnBtn.href = redirectUrl;
        returnBtn.style.display = 'flex';
    } else {
        returnBtn.style.display = 'none';
    }

    stopPolling();
}

async function fetchPaymentStatus() {
    const uuid = getUuidFromUrl();
    if (!uuid) {
        document.getElementById('status-text').textContent = 'Missing deposit UUID';
        return;
    }

    try {
        const response = await fetch(`${API_ENDPOINT}?deposit_uuid=${encodeURIComponent(uuid)}`);
        if (!response.ok) {
            throw new Error('Failed to fetch payment status');
        }
        const data = await response.json();
        updateUI(data);
    } catch (error) {
        console.error('Error fetching payment status:', error);
        document.getElementById('status-text').textContent = 'Error loading payment';
    }
}

async function checkPaymentStatus() {
    const checkBtn = document.getElementById('check-btn');
    checkBtn.classList.add('loading');

    await fetchPaymentStatus();

    checkBtn.classList.remove('loading');
}

function startPolling() {
    if (pollIntervalId) return;
    pollIntervalId = setInterval(fetchPaymentStatus, POLL_INTERVAL);
}

function stopPolling() {
    if (pollIntervalId) {
        clearInterval(pollIntervalId);
        pollIntervalId = null;
    }
}

function copyToClipboard() {
    const addressInput = document.getElementById('wallet-address');
    const copyText = document.getElementById('copy-text');
    
    navigator.clipboard.writeText(addressInput.value).then(() => {
        copyText.textContent = 'Copied!';
        showToast('Address copied!');
        
        setTimeout(() => {
            copyText.textContent = 'Copy';
        }, 2000);
    });
}

function showToast(message) {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.classList.add('show');
    
    setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}

function openQrModal() {
    const modal = document.getElementById('qr-modal');
    modal.classList.add('active');
}

function closeQrModal() {
    const modal = document.getElementById('qr-modal');
    modal.classList.remove('active');
}

document.addEventListener('DOMContentLoaded', () => {
    const copyBtn = document.getElementById('copy-btn');
    const checkBtn = document.getElementById('check-btn');
    const qrContainer = document.getElementById('qr-container');
    const qrModalClose = document.getElementById('qr-modal-close');
    const qrModal = document.getElementById('qr-modal');

    copyBtn.addEventListener('click', copyToClipboard);
    checkBtn.addEventListener('click', checkPaymentStatus);
    qrContainer.addEventListener('click', openQrModal);
    qrModalClose.addEventListener('click', closeQrModal);
    qrModal.addEventListener('click', (e) => {
        if (e.target === qrModal) {
            closeQrModal();
        }
    });

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            closeQrModal();
        }
    });

    fetchPaymentStatus().then(() => {
        startPolling();
    });
});
