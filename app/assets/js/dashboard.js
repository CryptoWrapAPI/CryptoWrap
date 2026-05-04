// Dashboard Functions

const COINS = [
    {
        id: 'monero',
        name: 'Monero',
        symbol: 'XMR',
        icon: '/assets/coins/monero100.png'
    },
    {
        id: 'litecoin',
        name: 'Litecoin',
        symbol: 'LTC',
        icon: '/assets/coins/litecoin100.png'
    }
];

document.addEventListener('DOMContentLoaded', () => {
    initializeDashboard();
});

function initializeDashboard() {
    // Logout button
    document.getElementById('logout-btn').addEventListener('click', handleLogout);

    // Refresh balances button
    document.getElementById('refresh-balances-btn').addEventListener('click', refreshAllBalances);

    // Load assets
    loadAssets();

    // Copy API Key button if it exists
    const copyApiBtn = document.querySelector('.btn-copy-api');
    if (copyApiBtn) {
        copyApiBtn.addEventListener('click', copyApiKey);
    }
}

function loadAssets() {
    const assetsList = document.getElementById('assets-list');
    assetsList.innerHTML = '';

    COINS.forEach(coin => {
        const assetItem = createAssetItem(coin);
        assetsList.appendChild(assetItem);
        loadCoinBalance(coin);
    });
}

function createAssetItem(coin) {
    const item = document.createElement('div');
    item.className = 'asset-item';
    item.id = `asset-${coin.id}`;
    item.innerHTML = `
        <div class="asset-info">
            <div class="asset-icon">
                <img src="${coin.icon}" alt="${coin.name}" />
                <div class="refresh-individual" id="refresh-individual-${coin.id}">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M23 4v6h-6"></path>
                        <path d="M1 20v-6h6"></path>
                        <path d="M3.51 9a9 9 0 0 1 14.85-3.36M20.49 15a9 9 0 0 1-14.85 3.36"></path>
                    </svg>
                </div>
            </div>
            <div class="asset-details">
                <div class="asset-name">${coin.name}</div>
                <div class="asset-amount-container">
                    <div class="asset-loading" id="loading-${coin.id}">
                        <span class="spinner"></span>
                        <span>Loading balance...</span>
                    </div>
                    <div class="asset-amount" id="amount-${coin.id}" style="display: none;"> --.-- ${coin.symbol}</div>
                    <div class="asset-error" id="error-${coin.id}" style="display: none;">
                        <span class="error-icon">⚠️</span>
                        <span class="error-text">Failed to load balance</span>
                    </div>
                </div>
            </div>
        </div>
        <button class="btn-withdraw" data-coin-id="${coin.id}" data-coin-symbol="${coin.symbol}">
            Withdraw
        </button>
    `;

    const withdrawBtn = item.querySelector('.btn-withdraw');
    withdrawBtn.addEventListener('click', () => openWithdrawModal(coin));

    const refreshBtn = item.querySelector('.refresh-individual');
    refreshBtn.addEventListener('click', () => refreshIndividualBalance(coin));

    return item;
}

async function loadCoinBalance(coin) {
    try {
        const response = await fetch(`/api/dashboard/balance?asset=${coin.id}`);
        
        if (!response.ok) {
            if (response.status === 403) {
                displayError(coin.id, 'Authentication required');
            } else {
                displayError(coin.id, 'Failed to fetch balance');
            }
            return;
        }
        
        const data = await response.json();
        displayBalance(coin.id, coin.symbol, data.balance);
    } catch (error) {
        console.error('Error fetching balance:', error);
        displayError(coin.id, 'Network error');
    }
}

function displayBalance(coinId, symbol, amount) {
    const loadingEl = document.getElementById(`loading-${coinId}`);
    const amountEl = document.getElementById(`amount-${coinId}`);
    const errorEl = document.getElementById(`error-${coinId}`);

    // Hide loading and error states
    if (loadingEl) loadingEl.style.display = 'none';
    if (errorEl) errorEl.style.display = 'none';
    
    // Show balance with proper formatting
    if (amountEl) {
        amountEl.textContent = `${amount.toFixed(8)} ${symbol}`;
        amountEl.style.display = 'block';
    }
}

function displayError(coinId, message) {
    const loadingEl = document.getElementById(`loading-${coinId}`);
    const amountEl = document.getElementById(`amount-${coinId}`);
    const errorEl = document.getElementById(`error-${coinId}`);
    const errorTextEl = document.getElementById(`error-${coinId}`).querySelector('.error-text');

    // Hide loading and balance states
    if (loadingEl) loadingEl.style.display = 'none';
    if (amountEl) amountEl.style.display = 'none';
    
    // Show error state
    if (errorEl) {
        if (message) {
            errorTextEl.textContent = message;
        }
        errorEl.style.display = 'flex';
    }
}

function refreshAllBalances() {
    const btn = document.getElementById('refresh-balances-btn');
    btn.classList.add('loading');
    btn.disabled = true;

    // Reset all balances to loading state
    COINS.forEach(coin => {
        const loadingEl = document.getElementById(`loading-${coin.id}`);
        const amountEl = document.getElementById(`amount-${coin.id}`);
        const errorEl = document.getElementById(`error-${coin.id}`);
        if (loadingEl) loadingEl.style.display = 'flex';
        if (amountEl) amountEl.style.display = 'none';
        if (errorEl) errorEl.style.display = 'none';
    });

    // Reload balances
    COINS.forEach(coin => {
        loadCoinBalance(coin);
    });

    // Re-enable button after animation completes
    setTimeout(() => {
        btn.classList.remove('loading');
        btn.disabled = false;
    }, 1500);
}

function refreshIndividualBalance(coin) {
    const refreshBtn = document.getElementById(`refresh-individual-${coin.id}`);
    refreshBtn.classList.add('loading');

    // Reset balance to loading state
    const loadingEl = document.getElementById(`loading-${coin.id}`);
    const amountEl = document.getElementById(`amount-${coin.id}`);
    const errorEl = document.getElementById(`error-${coin.id}`);
    if (loadingEl) loadingEl.style.display = 'flex';
    if (amountEl) amountEl.style.display = 'none';
    if (errorEl) errorEl.style.display = 'none';

    // Reload balance for specific coin
    loadCoinBalance(coin);

    // Re-enable button after animation completes
    setTimeout(() => {
        refreshBtn.classList.remove('loading');
    }, 1500);
}

function openWithdrawModal(coin) {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.id = `withdraw-modal-${coin.id}`;
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Withdraw ${coin.name}</h3>
                <button class="modal-close">&times;</button>
            </div>
            <div class="modal-body">
                <form class="withdraw-form" id="withdraw-form-${coin.id}">
                    <div class="form-group">
                        <label class="form-label">Destination Address</label>
                        <input 
                            type="text" 
                            class="form-input" 
                            name="destination_address" 
                            placeholder="Enter ${coin.name} address"
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">Amount (${coin.symbol})</label>
                        <input 
                            type="number" 
                            class="form-input" 
                            name="amount" 
                            placeholder="0.00000000"
                            step="0.00000001"
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">Auth Token</label>
                        <input 
                            type="password" 
                            class="form-input" 
                            name="auth_token" 
                            placeholder="Enter your auth token"
                            required
                        />
                    </div>

                    <div class="form-footer">
                        <button type="button" class="btn-modal-cancel">Cancel</button>
                        <button type="submit" class="btn-modal-send">Send</button>
                    </div>
                </form>
            </div>
        </div>
    `;
    // modal.innerHTML = `
    //     <div class="modal-content">
    //         <div class="modal-header">
    //             <h3>Withdraw ${coin.name}</h3>
    //             <button class="modal-close">&times;</button>
    //         </div>
    //         <div class="modal-body">
    //             <form class="withdraw-form" id="withdraw-form-${coin.id}">
    //                 <div class="form-group">
    //                     <label class="form-label">Destination Address</label>
    //                     <input 
    //                         type="text" 
    //                         class="form-input" 
    //                         name="destination_address" 
    //                         placeholder="Enter ${coin.name} address"
    //                         required
    //                     />
    //                 </div>

    //                 <div class="form-group">
    //                     <label class="form-label">Amount (${coin.symbol})</label>
    //                     <input 
    //                         type="number" 
    //                         class="form-input" 
    //                         name="amount" 
    //                         placeholder="0.00000000"
    //                         step="0.00000001"
    //                         required
    //                     />
    //                 </div>

    //                 <div class="form-group">
    //                     <label class="form-label">Network Fee</label>
    //                     <div class="form-static-value"> ~this should be dynamic based on tx weight ${coin.symbol}</div>
    //                 </div>

    //                 <div class="form-group">
    //                     <label class="form-label">Platform Fee</label>
    //                     <div class="form-static-value">0.1% (Free for API users)</div>
    //                 </div>

    //                 <div class="form-group">
    //                     <label class="form-label">Auth Token (Master Password)</label>
    //                     <input 
    //                         type="password" 
    //                         class="form-input" 
    //                         name="auth_token" 
    //                         placeholder="Enter your auth token"
    //                         required
    //                     />
    //                 </div>

    //                 <div class="form-group">
    //                     <label class="form-label">Wallet Address (Static)</label>
    //                     <div class="form-static-value">Reserved for future use</div>
    //                 </div>

    //                 <div class="form-footer">
    //                     <button type="button" class="btn-modal-cancel">Cancel</button>
    //                     <button type="submit" class="btn-modal-send">Send</button>
    //                 </div>
    //             </form>
    //         </div>
    //     </div>
    // `;

    document.body.appendChild(modal);
    addModalStyles();

    // Event listeners
    const closeBtn = modal.querySelector('.modal-close');
    const cancelBtn = modal.querySelector('.btn-modal-cancel');
    const form = modal.querySelector(`#withdraw-form-${coin.id}`);

    closeBtn.addEventListener('click', () => modal.remove());
    cancelBtn.addEventListener('click', () => modal.remove());
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });

    form.addEventListener('submit', (e) => handleWithdrawSubmit(e, coin, modal));
}

function handleWithdrawSubmit(e, coin, modal) {
    e.preventDefault();

    const form = e.target;
    const formData = new FormData(form);

    const withdrawData = {
        coin_id: coin.id,
        coin_symbol: coin.symbol,
        destination_address: formData.get('destination_address'),
        amount: parseFloat(formData.get('amount')),
        auth_token: formData.get('auth_token')
    };

    console.log('Submitting withdrawal:', withdrawData);

    // TODO: Replace with actual API call when endpoint is available
    // Example: POST /api/transactions/withdraw
    alert(`Withdrawal submitted for ${coin.name}. Please check the backend logs for verification.`);
    modal.remove();
}

function addModalStyles() {
    if (document.getElementById('modal-styles')) return;

    const style = document.createElement('style');
    style.id = 'modal-styles';
    style.textContent = `
        .modal-overlay {
            display: flex;
            align-items: center;
            justify-content: center;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.7);
            z-index: 1000;
            backdrop-filter: blur(4px);
        }

        .modal-content {
            background: #020204d6;
            border: 1px solid rgba(100, 150, 255, 0.2);
            border-radius: 12px;
            max-width: 500px;
            width: 90%;
            max-height: 90vh;
            overflow-y: auto;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7);
        }

        .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.5rem;
            border-bottom: 1px solid rgba(100, 150, 255, 0.2);
        }

        .modal-header h3 {
            color: #6a96ff;
            font-size: 1.25rem;
            margin: 0;
        }

        .modal-close {
            background: none;
            border: none;
            color: #888;
            font-size: 1.5rem;
            cursor: pointer;
            padding: 0;
            width: 30px;
            height: 30px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: color 0.3s ease;
        }

        .modal-close:hover {
            color: #e0e0e0;
        }

        .modal-body {
            padding: 1.5rem;
        }

        .withdraw-form {
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
        }

        .form-group {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }

        .form-label {
            font-size: 0.85rem;
            color: #888;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            font-weight: 500;
        }

        .form-input {
            padding: 0.75rem;
            background: #0f0f1a;
            border: 1px solid rgba(106, 150, 255, 0.2);
            border-radius: 6px;
            color: #e0e0e0;
            font-size: 0.95rem;
            font-family: inherit;
            transition: all 0.3s ease;
        }

        .form-input:focus {
            outline: none;
            border-color: #6a96ff;
            background: #0f0f1a;
            box-shadow: 0 0 0 2px rgba(106, 150, 255, 0.1);
        }

        .form-input::placeholder {
            color: #555;
        }

        .form-static-value {
            padding: 0.75rem;
            background: rgba(106, 150, 255, 0.05);
            border: 1px solid rgba(106, 150, 255, 0.15);
            border-radius: 6px;
            color: #aaa;
            font-size: 0.95rem;
        }

        .form-footer {
            display: flex;
            gap: 0.75rem;
            justify-content: flex-end;
            padding-top: 1rem;
            border-top: 1px solid rgba(100, 150, 255, 0.2);
        }

        .btn-modal-cancel,
        .btn-modal-send {
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.95rem;
            font-weight: 500;
            transition: all 0.3s ease;
        }

        .btn-modal-cancel {
            background: transparent;
            color: #6a96ff;
            border: 2px solid #6a96ff;
        }

        .btn-modal-cancel:hover {
            background: #6a96ff;
            color: white;
        }

        .btn-modal-send {
            background: linear-gradient(135deg, #3a5fc8 0%, #6a3ad9 100%);
            color: white;
        }

        .btn-modal-send:hover {
            background: linear-gradient(135deg, #6a3ad9 0%, #3a5fc8 100%);
        }

        .btn-modal-send:active,
        .btn-modal-cancel:active {
            transform: scale(0.98);
        }

        @media (max-width: 480px) {
            .modal-content {
                width: 95%;
            }

            .modal-header {
                padding: 1rem;
            }

            .modal-body {
                padding: 1rem;
            }

            .form-footer {
                flex-direction: column;
            }

            .btn-modal-cancel,
            .btn-modal-send {
                width: 100%;
            }
        }
    `;
    document.head.appendChild(style);
}


async function handleLogout() {
    if (confirm('Are you sure you want to logout?')) {
        try {
            const response = await fetch('/logout', {
                method: 'POST'
            });

            if (response.ok) {
                window.location.href = '/auth';
            } else {
                throw new Error('Logout failed');
            }
        } catch (error) {
            console.error('Error during logout:', error);
            window.location.href = '/auth';
        }
    }
}

function copyApiKey() {
    const apiKey = 'sk_live_8f7d3a2b9e1c4f5g6h8j9k2l3m4n5o6p';
    navigator.clipboard.writeText(apiKey).then(() => {
        const btn = event.target;
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 2000);
    }).catch(() => {
        alert('Failed to copy API key');
    });
}
