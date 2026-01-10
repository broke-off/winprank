$wsh = New-Object -ComObject WScript.Shell
while ($true) {
    $wsh.SendKeys("{CAPSLOCK}")
    $wsh.SendKeys("{NUMLOCK}")
    $wsh.SendKeys("{SCROLLLOCK}")
    Start-Sleep -Milliseconds 200
}