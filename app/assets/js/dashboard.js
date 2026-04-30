// Mock Dashboard Functions

document.addEventListener('DOMContentLoaded', () => {
    initializeDashboard();
});

function initializeDashboard() {
    // Logout button
    document.getElementById('logout-btn').addEventListener('click', handleLogout);

    // Quick Action buttons
    document.getElementById('withdraw-btn').addEventListener('click', () => handleAction('Withdraw'));
    document.getElementById('receive-btn').addEventListener('click', () => handleAction('Receive Payment'));
    document.getElementById('history-btn').addEventListener('click', () => handleAction('Transaction History'));
    document.getElementById('settings-btn').addEventListener('click', () => handleAction('Account Settings'));

    // Copy API Key button
    document.querySelector('.btn-copy-api').addEventListener('click', copyApiKey);
}

async function handleLogout() {
    if (confirm('Are you sure you want to logout?')) {
        try {
            const response = await fetch('/logout', {
                method: 'POST'
            });

            if (response.ok) {
                // Logout successful, redirect to login page
                window.location.href = '/auth';
            } else {
                throw new Error('Logout failed');
            }
        } catch (error) {
            console.error('Error during logout:', error);
            // Even if the API call fails, redirect to login
            window.location.href = '/auth';
        }
    }
}

function handleAction(actionName) {
    const actionMessages = {
        'Withdraw': {
            title: 'Withdraw Funds',
            message: 'Withdraw functionality would be implemented once withdrawal API endpoints are ready.',
            details: 'This would call: POST /api/transactions/withdraw\nWith parameters: amount, coin_type, destination_address'
        },
        'Receive Payment': {
            title: 'Receive Payment',
            message: 'Generate a new payment address for receiving cryptocurrency.',
            details: 'This would call: POST /api/accounts/generate-address\nWould return a unique deposit address for this virtual account'
        },
        'Transaction History': {
            title: 'Transaction History',
            message: 'View all transactions for this account.',
            details: 'This would call: GET /api/transactions\nWould display paginated list of all historical transactions'
        },
        'Account Settings': {
            title: 'Account Settings',
            message: 'Manage your account preferences and security settings.',
            details: 'This would allow you to: Update account info, change API keys, enable 2FA, manage webhook endpoints, etc.'
        }
    };

    const action = actionMessages[actionName];
    showActionModal(action);
}

function showActionModal(action) {
    const modal = document.createElement('div');
    modal.className = 'action-modal';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>${action.title}</h3>
                <button class="modal-close">&times;</button>
            </div>
            <div class="modal-body">
                <p>${action.message}</p>
                <pre class="modal-details">${action.details}</pre>
            </div>
            <div class="modal-footer">
                <button class="modal-btn-primary">Proceed</button>
                <button class="modal-btn-secondary">Cancel</button>
            </div>
        </div>
    `;

    document.body.appendChild(modal);

    // Add styles if not already present
    if (!document.getElementById('modal-styles')) {
        const style = document.createElement('style');
        style.id = 'modal-styles';
        style.textContent = `
            .action-modal {
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
            }

            .modal-close:hover {
                color: #e0e0e0;
            }

            .modal-body {
                padding: 1.5rem;
            }

            .modal-body p {
                color: #c0c0c0;
                margin-bottom: 1rem;
                line-height: 1.6;
            }

            .modal-details {
                background: #0f0f1a;
                border: 1px solid rgba(106, 150, 255, 0.2);
                border-radius: 6px;
                padding: 1rem;
                color: #6a96ff;
                font-family: 'Courier New', monospace;
                font-size: 0.8rem;
                overflow-x: auto;
                margin: 0;
            }

            .modal-footer {
                display: flex;
                gap: 0.75rem;
                padding: 1.5rem;
                border-top: 1px solid rgba(100, 150, 255, 0.2);
                justify-content: flex-end;
            }

            .modal-btn-primary,
            .modal-btn-secondary {
                padding: 0.6rem 1.2rem;
                border: none;
                border-radius: 6px;
                cursor: pointer;
                font-size: 0.9rem;
                transition: all 0.3s ease;
            }

            .modal-btn-primary {
                background: linear-gradient(135deg, #3a5fc8 0%, #6a3ad9 100%);
                color: white;
            }

            .modal-btn-primary:hover {
                background: linear-gradient(135deg, #6a3ad9 0%, #3a5fc8 100%);
            }

            .modal-btn-secondary {
                background: transparent;
                color: #6a96ff;
                border: 2px solid #6a96ff;
            }

            .modal-btn-secondary:hover {
                background: #6a96ff;
                color: white;
            }
        `;
        document.head.appendChild(style);
    }

    // Close button
    modal.querySelector('.modal-close').addEventListener('click', () => modal.remove());
    modal.querySelector('.modal-btn-secondary').addEventListener('click', () => modal.remove());

    // Proceed button
    modal.querySelector('.modal-btn-primary').addEventListener('click', () => {
        alert('This action would be processed by the API endpoint once implemented.');
        modal.remove();
    });

    // Close on backdrop click
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
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
