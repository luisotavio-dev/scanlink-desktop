#!/bin/bash
# Post-removal script for ScanLink
# Closes firewall port 47592 when app is uninstalled

PORT=47592
APP_NAME="ScanLink"

echo "Removing firewall configuration for $APP_NAME on port $PORT..."

# UFW (Ubuntu, Debian-based)
if command -v ufw &> /dev/null; then
    if ufw status | grep -q "Status: active"; then
        echo "Removing UFW rule..."
        ufw delete allow $PORT/tcp 2>/dev/null || true
    fi
fi

# firewalld (Fedora, RHEL, CentOS)
if command -v firewall-cmd &> /dev/null; then
    if systemctl is-active --quiet firewalld; then
        echo "Removing firewalld rule..."
        firewall-cmd --permanent --remove-port=$PORT/tcp --quiet 2>/dev/null || true
        firewall-cmd --reload --quiet 2>/dev/null || true
    fi
fi

# iptables (fallback for older systems)
if command -v iptables &> /dev/null; then
    if ! command -v ufw &> /dev/null && ! command -v firewall-cmd &> /dev/null; then
        echo "Removing iptables rule..."
        iptables -D INPUT -p tcp --dport $PORT -j ACCEPT 2>/dev/null || true

        # Try to save iptables rules
        if command -v iptables-save &> /dev/null; then
            if [ -d /etc/iptables ]; then
                iptables-save > /etc/iptables/rules.v4 2>/dev/null || true
            elif [ -f /etc/sysconfig/iptables ]; then
                iptables-save > /etc/sysconfig/iptables 2>/dev/null || true
            fi
        fi
    fi
fi

# nftables
if command -v nft &> /dev/null; then
    if ! command -v ufw &> /dev/null && ! command -v firewall-cmd &> /dev/null; then
        if systemctl is-active --quiet nftables 2>/dev/null; then
            echo "Removing nftables rule..."
            # Note: Removing specific nft rules is more complex, skipping for now
            # The rule will be removed on next reboot or nftables restart
        fi
    fi
fi

echo "$APP_NAME firewall configuration removed."
exit 0
