#!/bin/bash
# Post-installation script for ScanLink
# Opens firewall port 47592 for WebSocket server

PORT=47592
APP_NAME="ScanLink"

echo "Configuring firewall for $APP_NAME on port $PORT..."

# Detect which firewall is active and configure accordingly

# UFW (Ubuntu, Debian-based)
if command -v ufw &> /dev/null; then
    if ufw status | grep -q "Status: active"; then
        echo "Configuring UFW firewall..."
        ufw allow $PORT/tcp comment "$APP_NAME WebSocket Server" 2>/dev/null || true
        echo "UFW: Port $PORT opened successfully"
    fi
fi

# firewalld (Fedora, RHEL, CentOS)
if command -v firewall-cmd &> /dev/null; then
    if systemctl is-active --quiet firewalld; then
        echo "Configuring firewalld..."
        firewall-cmd --permanent --add-port=$PORT/tcp --quiet 2>/dev/null || true
        firewall-cmd --reload --quiet 2>/dev/null || true
        echo "firewalld: Port $PORT opened successfully"
    fi
fi

# iptables (fallback for older systems)
if command -v iptables &> /dev/null; then
    # Check if iptables is being used (not managed by ufw or firewalld)
    if ! command -v ufw &> /dev/null && ! command -v firewall-cmd &> /dev/null; then
        echo "Configuring iptables..."
        iptables -C INPUT -p tcp --dport $PORT -j ACCEPT 2>/dev/null || \
            iptables -I INPUT -p tcp --dport $PORT -j ACCEPT 2>/dev/null || true

        # Try to save iptables rules
        if command -v iptables-save &> /dev/null; then
            if [ -d /etc/iptables ]; then
                iptables-save > /etc/iptables/rules.v4 2>/dev/null || true
            elif [ -f /etc/sysconfig/iptables ]; then
                iptables-save > /etc/sysconfig/iptables 2>/dev/null || true
            fi
        fi
        echo "iptables: Port $PORT opened successfully"
    fi
fi

# nftables (newer systems)
if command -v nft &> /dev/null; then
    if ! command -v ufw &> /dev/null && ! command -v firewall-cmd &> /dev/null; then
        if systemctl is-active --quiet nftables 2>/dev/null; then
            echo "Configuring nftables..."
            nft add rule inet filter input tcp dport $PORT accept 2>/dev/null || true
            echo "nftables: Port $PORT opened successfully"
        fi
    fi
fi

echo "$APP_NAME firewall configuration completed."
exit 0
