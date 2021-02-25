using System;
using System.Runtime.InteropServices;

namespace Ib.Everything
{
    public class EverythingIpc
    {
        private IntPtr evWindow;

        EverythingIpc()
        {
            evWindow = FindWindow("EVERYTHING_TASKBAR_NOTIFICATION", IntPtr.Zero);
        }

        [DllImport("user32.dll", SetLastError = true)]
        static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    }
}