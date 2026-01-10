Add-Type -AssemblyName System.Windows.Forms
while($true) {
    $pos = [System.Windows.Forms.Cursor]::Position
    $pos.X += (Get-Random -Minimum -40 -Maximum 40)
    $pos.Y += (Get-Random -Minimum -40 -Maximum 40)
    [System.Windows.Forms.Cursor]::Position = $pos
    Start-Sleep -Milliseconds 50
}