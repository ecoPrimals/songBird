# Songbird Windows Service Deployment

**genomeBin Week 2:** Windows Service wrapper for production deployment

## 🎯 **Status: Phase 2 Planned**

Windows Service integration is planned for **genomeBin Phase 2** after full universal IPC integration (named pipes).

### **Current Windows Support:**

✅ **Binary:** `songbird.exe` builds successfully (49MB PE32+ executable)  
✅ **Platform:** Windows x64 (x86_64-pc-windows-gnu)  
✅ **IPC:** TCP localhost fallback (functional)  
📝 **Service:** Wrapper planned for Phase 2

---

## 🚀 **Current Deployment (Phase 1)**

### Manual Execution:

```powershell
# Download or build songbird.exe
# Place in: C:\Program Files\Songbird\songbird.exe

# Run directly (console mode)
.\songbird.exe

# Or with environment variables
$env:SONGBIRD_FAMILY_ID = "my-game"
$env:RUST_LOG = "info"
.\songbird.exe
```

### Background Execution (PowerShell):

```powershell
# Start in background
Start-Process -FilePath ".\songbird.exe" -WindowStyle Hidden

# Find process
Get-Process songbird

# Stop process
Stop-Process -Name songbird
```

---

## 📋 **Phase 2: Windows Service (Planned)**

### **Planned Features:**

✅ **Windows Service:**
- Automatic start on boot
- Service Control Manager integration
- Event Log integration
- Service recovery policies

✅ **Named Pipes IPC:**
- Native Windows IPC (replacing TCP fallback)
- `\\.\pipe\songbird-{family}` format
- Better performance and security

✅ **Windows Integration:**
- Performance counters
- Windows Firewall rules
- Windows Defender exclusions
- UAC elevation prompts

### **Planned Tools:**

```
deployment/windows-service/
├── songbird-service.exe          # Service wrapper (to be created)
├── install-service.ps1           # Installation script
├── uninstall-service.ps1         # Removal script
├── config-firewall.ps1           # Firewall configuration
└── README.md                     # This file
```

### **Installation (Future):**

```powershell
# Install service (requires Administrator)
.\install-service.ps1 -FamilyID "my-game"

# Start service
Start-Service Songbird

# Check status
Get-Service Songbird

# Stop service
Stop-Service Songbird

# Uninstall service
.\uninstall-service.ps1
```

---

## 🔧 **Workaround: Task Scheduler (Current)**

Use Windows Task Scheduler for auto-start (until Phase 2):

### Create Task:

```powershell
# Create scheduled task (runs at startup)
$action = New-ScheduledTaskAction -Execute "C:\Program Files\Songbird\songbird.exe"
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

Register-ScheduledTask -TaskName "Songbird" -Action $action -Trigger $trigger -Principal $principal -Settings $settings
```

### Manage Task:

```powershell
# Start task
Start-ScheduledTask -TaskName "Songbird"

# Stop task
Stop-ScheduledTask -TaskName "Songbird"

# Remove task
Unregister-ScheduledTask -TaskName "Songbird" -Confirm:$false
```

---

## 🌐 **Firewall Configuration**

### Allow Songbird Through Firewall:

```powershell
# Allow inbound (if needed for P2P)
New-NetFirewallRule -DisplayName "Songbird Network Orchestrator" `
  -Direction Inbound `
  -Program "C:\Program Files\Songbird\songbird.exe" `
  -Action Allow `
  -Profile Any

# Allow outbound
New-NetFirewallRule -DisplayName "Songbird Network Orchestrator" `
  -Direction Outbound `
  -Program "C:\Program Files\Songbird\songbird.exe" `
  -Action Allow `
  -Profile Any
```

---

## 📊 **Current Limitations (Phase 1)**

### ⚠️ **TCP Fallback IPC:**
- Uses TCP localhost instead of named pipes
- Slightly less efficient than Unix sockets
- Security relies on localhost binding

### ⚠️ **No Service Integration:**
- Must be started manually or via Task Scheduler
- No automatic recovery
- No Event Log integration

### ⚠️ **Limited Monitoring:**
- No Performance Counters
- Basic logging only
- No Windows-specific health checks

**Note:** All limitations will be addressed in Phase 2!

---

## 🔄 **Migration Path (Phase 1 → Phase 2)**

When Phase 2 is released:

1. **Stop Current Instance:**
   ```powershell
   Stop-Process -Name songbird
   # Or: Stop-ScheduledTask -TaskName "Songbird"
   ```

2. **Install Service Wrapper:**
   ```powershell
   .\install-service.ps1
   ```

3. **Migrate Configuration:**
   - Service reads from same locations
   - Automatic migration of settings

4. **Start Service:**
   ```powershell
   Start-Service Songbird
   ```

---

## 📚 **Related Documentation**

- [systemd Deployment](../systemd/README.md) (Linux)
- [USB Live Spore](../usb-live-spore/README.md) (Portable)
- [genomeBin Evolution](../../GENOMEBIN_EVOLUTION_ANALYSIS_JAN_31_2026.md)
- [Windows Build Success](../../GENOMEBIN_WEEK1_VICTORY_JAN_31_2026.md)

---

## 📝 **Phase 2 Roadmap**

### Week 1-2: Named Pipes IPC
- Implement Windows named pipes in `songbird-universal-ipc`
- Replace TCP fallback with native pipes
- Testing and validation

### Week 3: Service Wrapper
- Create `songbird-service.exe` wrapper
- Service Control Manager integration
- Installation/uninstall scripts

### Week 4: Windows Integration
- Event Log integration
- Performance counters
- Firewall automation
- Auto-update support

---

**Status:** ⏸️ Phase 2 Planned  
**Current:** TCP fallback (functional)  
**Binary:** 49MB PE32+ executable  
**Target:** x86_64-pc-windows-gnu  
**genomeBin:** Week 2 - Windows deployment (Phase 1 baseline)
