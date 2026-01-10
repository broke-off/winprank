$code = @"
    [DllImport("user32.dll")]
    public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);
"@
$type = Add-Type -MemberDefinition $code -Name "Win32SendMessage" -Namespace Win32Functions -PassThru

$type::SendMessage(0xFFFF, 0x0112, 0xF170, -1)