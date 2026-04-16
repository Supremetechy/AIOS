# Minimal sys module for embedded Python
# Provides basic system functionality

version_info = (3, 12, 0, 'final', 0)
version = '3.12.0 (AI-OS embedded)'
platform = 'aios'
executable = '/aios/bin/python'

# Module search paths (set by kernel)
path = [
    '/aios/lib/python3.12',
    '/aios/lib/python3.12/site-packages',
    '/aios/models',
    '/aios/scripts',
    '.',
]

# Standard streams (connected to kernel)
class StdStream:
    def write(self, text):
        # Calls kernel print
        import aios_kernel
        aios_kernel.print(text)
    
    def flush(self):
        pass

stdout = StdStream()
stderr = StdStream()
stdin = None  # TODO: Implement keyboard input

def exit(code=0):
    """Exit the Python interpreter"""
    import aios_kernel
    aios_kernel.syscall(0, code)  # SYS_EXIT

def argv():
    """Get command line arguments"""
    return ['python']
