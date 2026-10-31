import keyboard
from time import sleep
import itertools

try:
    from PyQt5.QtWidgets import QApplication
except ImportError:
    QApplication = None

lista_f = [f'f{i}' for i in range(1, 25)]
lista_letras_mai = [chr(i) for i in range(ord('A'), ord('Z')+1)]
lista_letras_min = [chr(i+32) for i in range(ord('A'), ord('Z')+1)]
lista_letras = list(itertools.chain(lista_letras_min, lista_letras_mai))
lista_nums = [str(i) for i in range(10)]


def format_key_combination(keys):
    """Formata a lista de teclas para exibição amigável (ex: F8, Ctrl+F8, Ctrl+Alt+T)."""
    if not keys:
        return ""
    mod_order = {'ctrl': 0, 'alt': 1, 'shift': 2, 'windows': 3, 'win': 3}
    modifiers = [k for k in keys if k.lower() in mod_order]
    non_modifiers = [k for k in keys if k.lower() not in mod_order]
    sorted_mods = sorted(modifiers, key=lambda k: mod_order.get(k.lower(), 10))

    formatted_parts = []
    for mod in sorted_mods:
        m = mod.lower()
        if m == 'ctrl':
            formatted_parts.append('Ctrl')
        elif m == 'alt':
            formatted_parts.append('Alt')
        elif m == 'shift':
            formatted_parts.append('Shift')
        elif m in ('win', 'windows'):
            formatted_parts.append('Win')
        else:
            formatted_parts.append(mod.capitalize())

    for key in non_modifiers:
        kl = key.lower()
        if kl.startswith('f') and kl[1:].isdigit():
            formatted_parts.append(kl.upper())
        elif len(kl) == 1:
            formatted_parts.append(kl.upper())
        else:
            formatted_parts.append(key.capitalize())

    return '+'.join(formatted_parts)


class KListen():
    def __init__(self, UI=None, combo="GTRAY"):
        self.UI = UI
        self.hook = None
        self.main(combo)

    def main(self, combo="GTRAY"):
        self.keys = []
        self.retry = 200
        self.combo = combo
        self.flag_sair = False
        self.activated = True
        self.captured_key = ""

    def set_activated(self, activated):
        self.activated = bool(activated)

    def set_combo(self, combo):
        self.combo = combo

    def start_listening(self):
        self.stop_listening()
        self.main(self.combo)
        try:
            self.hook = keyboard.on_press(self.record_key)
        except Exception:
            self.hook = None

    def stop_listening(self):
        if self.hook is not None:
            try:
                keyboard.unhook(self.hook)
            except Exception:
                pass
            self.hook = None
        self.activated = False

    def record_key(self, event):
        if not self.activated:
            return

        if getattr(event, 'event_type', None) != 'down':
            return

        event_name = getattr(event, 'name', '')
        if not event_name:
            return

        raw_name = str(event_name).lower()
        if raw_name == 'esc':
            self.flag_sair = True
            return

        if 'ctrl' in raw_name:
            norm_name = 'ctrl'
        elif 'alt' in raw_name:
            norm_name = 'alt'
        elif 'shift' in raw_name:
            norm_name = 'shift'
        elif 'win' in raw_name:
            norm_name = 'windows'
        else:
            norm_name = raw_name

        if norm_name not in self.keys:
            self.keys.append(norm_name)

        if norm_name in ('ctrl', 'alt', 'shift', 'windows'):
            return

        has_modifiers = any(k in ('ctrl', 'alt', 'shift', 'windows') for k in self.keys)
        is_f_key = norm_name in lista_f or (norm_name.startswith('f') and norm_name[1:].isdigit())
        is_char_or_num = norm_name in lista_letras or norm_name in lista_nums or len(norm_name) == 1

        if is_f_key or has_modifiers:
            self.captured_key = format_key_combination(self.keys)
            self.flag_sair = True
        elif not has_modifiers and is_char_or_num:
            self.keys = []

    def run(self):
        self.start_listening()
        try:
            while not self.flag_sair and self.retry > 0:
                sleep(0.05)
                self.retry -= 1
                if QApplication and QApplication.instance():
                    QApplication.processEvents()
        finally:
            self.stop_listening()
        return self.captured_key


if __name__ == '__main__':
    kl = KListen()
    res = kl.run()
    print(f"Captured: {res}")
