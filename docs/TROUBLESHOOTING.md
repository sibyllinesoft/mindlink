# MindLink Troubleshooting Guide

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Authentication Issues](#authentication-issues)
3. [Connection Problems](#connection-problems)
4. [API Errors](#api-errors)
5. [Performance Issues](#performance-issues)
6. [System-Specific Problems](#system-specific-problems)
7. [Advanced Troubleshooting](#advanced-troubleshooting)
8. [Log Analysis](#log-analysis)
9. [Getting Support](#getting-support)

## Quick Diagnostics

### System Health Check

First, run the built-in diagnostic tools to identify common issues:

**1. Connection Status Check**
```bash
# Right-click MindLink tray icon → "Connection Status"
# Or check the dashboard health endpoint
curl http://localhost:3001/health
```

Expected healthy response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "services": {
    "auth": "authenticated",
    "server": "running", 
    "tunnel": "connected"
  }
}
```

**2. Service Status Overview**
```bash
# Check if MindLink is running
ps aux | grep mindlink  # Linux/macOS
tasklist | findstr mindlink  # Windows

# Check if port is in use
netstat -tlnp | grep :3001  # Linux
netstat -an | findstr :3001  # Windows
lsof -i :3001  # macOS
```

**3. Quick Configuration Check**
```bash
# Verify configuration file exists and is valid
# Linux: ~/.config/com.mindlink.mindlink/config.json
# macOS: ~/Library/Application Support/com.mindlink.mindlink/config.json  
# Windows: %APPDATA%\com.mindlink.mindlink\config.json

cat ~/.config/com.mindlink.mindlink/config.json | jq .  # Validate JSON
```

### Error Code Quick Reference

| Error Code | Meaning | Quick Fix |
|------------|---------|-----------|
| `AUTH_001` | Authentication expired | Right-click tray → "Login & Serve" |
| `NET_001` | Cannot connect to ChatGPT | Check internet connection |
| `TUNNEL_001` | Tunnel creation failed | Check firewall settings |
| `SERVER_001` | Port already in use | Change port in settings |
| `CONFIG_001` | Invalid configuration | Reset to defaults |

## Authentication Issues

### Problem: "Authentication Required" Error

**Symptoms:**
- API calls return 401 Unauthorized
- Tray icon shows red status
- Dashboard shows "Not authenticated"

**Solutions:**

**1. Re-authenticate**
```bash
# Method 1: Through system tray
# Right-click MindLink icon → "Login & Serve"

# Method 2: Through dashboard
# Open http://localhost:3001/dashboard → Click "Authenticate"

# Method 3: Clear stored credentials and re-authenticate
rm ~/.config/com.mindlink.mindlink/auth_tokens.json  # Linux
# Then restart MindLink and authenticate again
```

**2. Clear browser cache and cookies**
```bash
# The OAuth flow may be cached incorrectly
# Clear cookies for auth.openai.com in your browser
# Try authentication in incognito/private mode
```

**3. Check ChatGPT account status**
- Verify your ChatGPT Plus/Pro subscription is active
- Log into ChatGPT directly to ensure account is not suspended
- Check for any security restrictions on your account

### Problem: OAuth Flow Gets Stuck

**Symptoms:**
- Browser opens but doesn't redirect back
- "Waiting for authentication..." never completes
- Browser shows OAuth error page

**Solutions:**

**1. Manual OAuth completion**
```bash
# If you see a URL like this in the browser:
# http://localhost:8080/callback?code=AUTH_CODE_HERE
# Copy the code and complete manually through dashboard
```

**2. Try different browser**
```bash
# Default browser may have issues
# Try Chrome, Firefox, Safari, or Edge
# Disable browser extensions temporarily
```

**3. Check firewall and antivirus**
```bash
# Ensure localhost connections are allowed
# Temporarily disable firewall to test
# Add MindLink to antivirus exceptions
```

### Problem: Token Refresh Failures

**Symptoms:**
- Frequent re-authentication requests
- "Token expired" errors every few minutes
- Authentication works but doesn't persist

**Solutions:**

**1. Check system clock**
```bash
# Incorrect system time causes token validation issues
timedatectl status  # Linux
date  # macOS/Linux
# Sync with NTP if time is incorrect
```

**2. Clear and regenerate tokens**
```bash
# Stop MindLink
# Delete token storage
rm ~/.config/com.mindlink.mindlink/auth_tokens.json
# Restart and re-authenticate
```

**3. Check network stability**
```bash
# Unstable connections can interrupt refresh
ping -c 10 auth.openai.com
# Consider using ethernet instead of WiFi
```

## Connection Problems

### Problem: Cannot Create Tunnel

**Symptoms:**
- "Failed to create tunnel" error
- Tray icon stuck in "connecting" state
- No public URL generated

**Solutions:**

**1. Check internet connectivity**
```bash
# Test basic connectivity
ping -c 4 google.com

# Test Cloudflare specifically
ping -c 4 cloudflare.com
nslookup cloudflare.com

# Test HTTPS connectivity
curl -I https://api.cloudflare.com/client/v4/zones
```

**2. Firewall and network configuration**
```bash
# Check if outbound HTTPS is blocked
telnet cloudflare.com 443

# Common firewall solutions:
# Windows: Allow MindLink through Windows Firewall
# macOS: System Preferences → Security & Privacy → Firewall
# Linux: Check iptables/ufw rules
```

**3. Try different tunnel type**
```json
// Change tunnel configuration in settings
{
  "tunnel": {
    "type": "quick",  // or "named"
    "custom_domain": null,
    "health_check_interval": 30
  }
}
```

**4. Corporate network troubleshooting**
```bash
# Corporate firewalls often block tunnel services
# Try connecting from personal network to verify
# Contact IT department about whitelist requirements

# Required domains for whitelisting:
# - *.trycloudflare.com
# - *.cloudflare.com  
# - api.cloudflare.com
```

### Problem: Local Server Won't Start

**Symptoms:**
- "Port already in use" error
- "Failed to bind to address" error
- API calls to localhost:3001 fail

**Solutions:**

**1. Find and kill conflicting process**
```bash
# Linux/macOS
lsof -i :3001
kill -9 PID_FROM_ABOVE

# Windows
netstat -ano | findstr :3001
taskkill /PID PID_FROM_ABOVE /F
```

**2. Change port configuration**
```json
{
  "server": {
    "port": 3002,  // Try different port
    "host": "127.0.0.1"
  }
}
```

**3. Check permissions**
```bash
# Ports below 1024 require admin privileges
# Try port above 1024
# Run as administrator if necessary (Windows)
```

### Problem: Tunnel URL Not Accessible

**Symptoms:**
- Tunnel created but URL returns connection timeout
- Works locally but not through tunnel
- Some requests work, others don't

**Solutions:**

**1. Wait for propagation**
```bash
# Tunnels can take 30-60 seconds to propagate globally
# Test with curl after waiting
curl -I https://your-tunnel.trycloudflare.com/health
```

**2. Check tunnel health**
```bash
# Through dashboard: Monitor tunnel status
# Through API: Check /health endpoint
# Restart tunnel if unhealthy
```

**3. DNS and routing issues**
```bash
# Test DNS resolution
nslookup your-tunnel.trycloudflare.com
dig your-tunnel.trycloudflare.com

# Test from different networks
# Mobile hotspot, different ISP, etc.
```

## API Errors

### Problem: "Model not available" Errors

**Symptoms:**
- 400 Bad Request with model error
- Specific models like "gpt-5" not working
- Models list appears empty

**Solutions:**

**1. Check available models**
```bash
curl http://localhost:3001/v1/models \
  -H "Authorization: Bearer any-key"
```

**2. Verify ChatGPT subscription**
```bash
# Log into ChatGPT directly
# Verify which models you have access to
# Check subscription status and billing
```

**3. Use correct model names**
```json
{
  "model": "gpt-4",  // Instead of "gpt-4-turbo"
  "messages": [...]
}
```

**4. Model availability by subscription**
- **ChatGPT Plus**: gpt-4, gpt-3.5-turbo
- **ChatGPT Pro**: gpt-5, gpt-4o, gpt-4, gpt-3.5-turbo
- Check OpenAI's latest model availability

### Problem: Rate Limiting Errors

**Symptoms:**
- HTTP 429 Too Many Requests
- "Rate limit exceeded" messages
- Requests timing out during busy periods

**Solutions:**

**1. Implement retry logic**
```python
import time
import random

def api_call_with_retry(func, max_retries=3):
    for attempt in range(max_retries):
        try:
            return func()
        except RateLimitError:
            if attempt < max_retries - 1:
                delay = (2 ** attempt) + random.uniform(0, 1)
                time.sleep(delay)
                continue
            raise
```

**2. Monitor usage patterns**
```bash
# Check dashboard for request patterns
# Spread requests over time instead of bursts
# Consider implementing request queuing
```

**3. Upgrade subscription if needed**
- ChatGPT Pro has higher rate limits
- Check OpenAI's current rate limit documentation

### Problem: Streaming Responses Not Working

**Symptoms:**
- Streaming parameter ignored
- Responses arrive all at once
- Connection drops during streaming

**Solutions:**

**1. Verify streaming is enabled**
```python
response = client.chat.completions.create(
    model="gpt-4",
    messages=[...],
    stream=True,  # Ensure this is True
    max_tokens=1000
)

for chunk in response:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

**2. Check client library support**
```bash
# Update to latest OpenAI client library
pip install --upgrade openai
npm update openai
```

**3. Test with curl**
```bash
curl -N http://localhost:3001/v1/chat/completions \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Count to 10"}],
    "stream": true
  }'
```

## Performance Issues

### Problem: Slow Response Times

**Symptoms:**
- High latency (>5 seconds for simple requests)
- Timeouts on longer requests
- Dashboard shows high response times

**Solutions:**

**1. Check network performance**
```bash
# Test latency to OpenAI
ping -c 10 api.openai.com

# Test tunnel latency
curl -w "@curl-format.txt" https://your-tunnel.trycloudflare.com/health

# curl-format.txt content:
# time_total: %{time_total}\n
# time_namelookup: %{time_namelookup}\n
# time_connect: %{time_connect}\n
```

**2. Optimize request parameters**
```json
{
  "model": "gpt-3.5-turbo",  // Faster than gpt-4
  "max_tokens": 500,         // Reduce if appropriate
  "temperature": 0.7,
  "stream": true             // For perceived performance
}
```

**3. Use local endpoint when possible**
```bash
# Local is always faster than tunnel
# Use http://localhost:3001 instead of tunnel URL
# Reserve tunnel for external/mobile access
```

**4. Monitor system resources**
```bash
# Check CPU and memory usage
top  # Linux/macOS
Task Manager  # Windows

# Restart MindLink if memory usage is high
```

### Problem: High Memory Usage

**Symptoms:**
- MindLink using >500MB RAM
- System becoming slow
- Out of memory errors

**Solutions:**

**1. Restart service regularly**
```bash
# Through system tray: "Restart Service"
# Or kill and restart process
```

**2. Reduce logging level**
```bash
# Set environment variable
export RUST_LOG=warn  # Instead of debug
# Or change in settings if available
```

**3. Clear cache and logs**
```bash
# Clear log files
rm ~/.local/share/com.mindlink.mindlink/logs/*  # Linux
rm ~/Library/Logs/com.mindlink.mindlink/*      # macOS
del %LOCALAPPDATA%\com.mindlink.mindlink\logs\*  # Windows

# Clear cache through dashboard
# Settings → Privacy → Clear Cache
```

## System-Specific Problems

### Windows Issues

**Problem: Windows Defender False Positive**
```powershell
# Add MindLink to exclusions
Add-MpPreference -ExclusionPath "C:\Program Files\MindLink\"
Add-MpPreference -ExclusionProcess "mindlink.exe"

# Or through Windows Security GUI:
# Virus & threat protection → Exclusions → Add exclusion
```

**Problem: MSI Installation Fails**
```cmd
# Run as administrator
# Check Windows Installer service
sc query msiserver

# Manual installation log
msiexec /i MindLink.msi /l*v install.log

# Check install.log for specific errors
```

**Problem: Service Doesn't Start on Boot**
```cmd
# Check startup entries
msconfig

# Or through registry
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run"

# Add manual entry if needed
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "MindLink" /t REG_SZ /d "\"C:\Program Files\MindLink\mindlink.exe\" --minimized"
```

### macOS Issues

**Problem: "App can't be opened" Security Warning**
```bash
# Allow app through Gatekeeper
sudo spctl --master-disable  # Temporarily
# Or right-click app → Open → Open anyway

# Check quarantine attribute
xattr -l /Applications/MindLink.app
# Remove quarantine if present
sudo xattr -rd com.apple.quarantine /Applications/MindLink.app
```

**Problem: Keychain Access Denied**
```bash
# Reset keychain permissions
security unlock-keychain ~/Library/Keychains/login.keychain

# If still issues, recreate keychain
# Keychain Access → File → New Keychain → Create new default keychain
```

**Problem: Tunnel Issues on macOS Monterey+**
```bash
# Check system firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Allow MindLink through firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /Applications/MindLink.app/Contents/MacOS/MindLink
```

### Linux Issues

**Problem: Missing System Dependencies**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0 libayatana-appindicator3-1

# Fedora/CentOS
sudo dnf install webkit2gtk3 gtk3 libappindicator-gtk3

# Arch Linux
sudo pacman -S webkit2gtk gtk3 libappindicator-gtk3
```

**Problem: Wayland Compatibility**
```bash
# Force X11 mode if Wayland causes issues
export GDK_BACKEND=x11
mindlink

# Or set permanently in .bashrc
echo 'export GDK_BACKEND=x11' >> ~/.bashrc
```

**Problem: AppImage Won't Execute**
```bash
# Make executable
chmod +x mindlink-*.AppImage

# Check FUSE availability
/usr/bin/fusermount --version

# Install FUSE if missing
sudo apt install fuse  # Ubuntu/Debian
sudo dnf install fuse  # Fedora
```

## Advanced Troubleshooting

### Debug Mode

**Enable comprehensive logging:**
```bash
# Linux/macOS
export RUST_LOG=debug
export RUST_BACKTRACE=1
/path/to/mindlink

# Windows (PowerShell)
$env:RUST_LOG="debug"
$env:RUST_BACKTRACE="1"
.\mindlink.exe

# Component-specific debugging
export RUST_LOG=mindlink::managers::auth_manager=debug
export RUST_LOG=mindlink::managers::server_manager=debug
export RUST_LOG=mindlink::managers::tunnel_manager=debug
```

### Network Debugging

**1. Packet capture for API analysis**
```bash
# Capture API traffic (requires root/admin)
sudo tcpdump -i any -w mindlink-traffic.pcap host api.openai.com
# Analyze with Wireshark

# Alternative: Use mitmproxy for HTTPS inspection
pip install mitmproxy
mitmproxy --listen-port 8080 --set confdir=~/.mitmproxy
```

**2. DNS resolution debugging**
```bash
# Test DNS resolution
dig api.openai.com
nslookup api.openai.com

# Test with different DNS servers
dig @8.8.8.8 api.openai.com
dig @1.1.1.1 api.openai.com

# Check /etc/resolv.conf (Linux/macOS)
cat /etc/resolv.conf
```

### Configuration Debugging

**1. Validate configuration files**
```bash
# Check JSON validity
python -m json.tool ~/.config/com.mindlink.mindlink/config.json

# Check for hidden characters
hexdump -C config.json | head
```

**2. Reset to default configuration**
```bash
# Backup current config
cp ~/.config/com.mindlink.mindlink/config.json config.json.backup

# Remove config to force defaults
rm ~/.config/com.mindlink.mindlink/config.json

# Restart MindLink - it will recreate default config
```

### Performance Profiling

**1. CPU profiling (development builds)**
```bash
# Linux - use perf
perf record --call-graph dwarf ./mindlink
perf report

# macOS - use Instruments
# Open Xcode → Developer Tools → Instruments
# Choose Time Profiler template
```

**2. Memory profiling**
```bash
# Linux - use valgrind
valgrind --tool=massif --detailed-freq=1 ./mindlink
ms_print massif.out.* > memory-report.txt

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full ./mindlink
```

## Log Analysis

### Log Locations

```bash
# Linux
~/.local/share/com.mindlink.mindlink/logs/
/tmp/com.mindlink.mindlink/

# macOS  
~/Library/Logs/com.mindlink.mindlink/
~/Library/Application Support/com.mindlink.mindlink/logs/

# Windows
%LOCALAPPDATA%\com.mindlink.mindlink\logs\
%TEMP%\com.mindlink.mindlink\
```

### Log Analysis Commands

**1. Search for specific errors**
```bash
# Find authentication errors
grep -r "auth" ~/.local/share/com.mindlink.mindlink/logs/

# Find network errors
grep -r -i "network\|connection\|timeout" logs/

# Find recent errors (last hour)
find logs/ -name "*.log" -newermt "1 hour ago" -exec grep -l "ERROR" {} \;
```

**2. Parse structured logs**
```bash
# Extract JSON logs and pretty print
cat mindlink.log | jq 'select(.level == "ERROR")'

# Count errors by type
cat mindlink.log | jq -r '.message' | sort | uniq -c | sort -nr

# Timeline of events
cat mindlink.log | jq -r '[.timestamp, .level, .message] | @tsv' | sort
```

### Common Error Patterns

**Authentication Loop**
```
Pattern: "Authentication required" → "Login successful" → "Authentication required" (repeating)
Cause: Token refresh failing
Solution: Clear tokens and re-authenticate
```

**Connection Timeouts**
```  
Pattern: Multiple "Connection timeout" or "Connection refused" errors
Cause: Network connectivity or firewall issues
Solution: Check network settings and firewall rules
```

**Port Conflicts**
```
Pattern: "Address already in use" or "Failed to bind"
Cause: Another service using the same port
Solution: Change port or kill conflicting process
```

## Getting Support

### Before Contacting Support

1. **Check this troubleshooting guide** for your specific issue
2. **Enable debug logging** and reproduce the issue
3. **Collect system information**:
   ```bash
   # System info
   uname -a  # Linux/macOS
   systeminfo  # Windows
   
   # MindLink version
   mindlink --version
   
   # Configuration (remove sensitive data)
   cat config.json
   ```

### Information to Include

**Essential Information:**
- MindLink version
- Operating system and version
- Error messages (exact text)
- Steps to reproduce
- Expected vs actual behavior

**Debug Information:**
- Relevant log excerpts (with timestamps)
- Configuration file (sanitized)
- Network test results
- System resource usage

### Support Channels

**1. GitHub Issues** (Bug reports and feature requests)
- URL: https://github.com/yourusername/mindlink/issues
- Include: System info, logs, reproduction steps
- Search existing issues first

**2. GitHub Discussions** (Questions and community support)  
- URL: https://github.com/yourusername/mindlink/discussions
- Best for: General questions, usage help, troubleshooting

**3. Community Discord** (Real-time chat support)
- URL: https://discord.gg/mindlink
- Best for: Quick questions, community help

**4. Email Support** (Enterprise and priority issues)
- Email: support@mindlink.dev
- Response time: 1-2 business days
- Include all debug information

### Creating Good Bug Reports

**Template:**
```markdown
## Bug Report

**MindLink Version:** 1.0.0
**OS:** Ubuntu 22.04 LTS
**Browser:** Chrome 120.0

### Expected Behavior
Authentication should complete successfully after OAuth flow.

### Actual Behavior  
Browser redirects back but MindLink shows "Authentication failed" error.

### Steps to Reproduce
1. Right-click tray icon → "Login & Serve"  
2. Complete OAuth flow in browser
3. Browser shows success page
4. MindLink still shows not authenticated

### Error Messages
```
2024-01-15 10:30:15 ERROR auth_manager: OAuth token exchange failed: invalid_grant
```

### Environment Details
- Network: Corporate network with proxy
- Firewall: Windows Defender enabled
- Antivirus: McAfee Corporate

### Additional Context
- Worked fine on home network
- Other OAuth apps work correctly
- No browser extensions affecting the flow
```

### Emergency Support (Critical Issues)

For critical issues affecting production deployments:

1. **Mark issue as critical** in GitHub with `critical` label
2. **Email support immediately** with "CRITICAL" in subject
3. **Include complete debug info** and impact assessment
4. **Provide contact information** for immediate response

**Critical issue examples:**
- Security vulnerabilities
- Data loss or corruption
- Service completely unavailable
- Authentication bypassed

Remember: Most issues can be resolved by following this troubleshooting guide. The community is also very helpful for common problems and usage questions.