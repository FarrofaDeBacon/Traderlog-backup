import tkinter as tk
from tkinter import ttk, messagebox
import hmac
import hashlib
import json
import base64
import datetime

# --- CONFIG ---
SECRET_KEY = "TRADERLOGPRO_SECRET_KEY_2026"
VERSION = "v1"
PREFIX = "TLP"

# --- COLORS (TraderLogPro Dark Palette) ---
BG_COLOR = "#09090b"  # Zinc 950
CARD_COLOR = "#18181b"  # Zinc 900
TEXT_COLOR = "#fafafa"  # Zinc 50
SUBTEXT_COLOR = "#a1a1aa"  # Zinc 400
PRIMARY_COLOR = "#3b82f6"  # Blue 500
PRIMARY_HOVER = "#2563eb"  # Blue 600
BORDER_COLOR = "#27272a"  # Zinc 800

class ModernLicenseGenerator:
    def __init__(self, root):
        self.root = root
        self.root.title("TraderLogPro | License Generator")
        self.root.geometry("520x580")
        self.root.configure(bg=BG_COLOR)
        
        # Set fonts
        self.font_main = ("Segoe UI", 10)
        self.font_bold = ("Segoe UI", 10, "bold")
        self.font_header = ("Segoe UI", 16, "bold")
        self.font_mono = ("Consolas", 9)

        self.setup_styles()
        self.is_lifetime = tk.BooleanVar(value=False)
        self.create_widgets()

    def setup_styles(self):
        style = ttk.Style()
        style.theme_use('clam')
        
        # Configure Frame
        style.configure("TFrame", background=BG_COLOR)
        style.configure("Card.TFrame", background=CARD_COLOR, borderwidth=1, relief="solid")
        
        # Labels
        style.configure("TLabel", background=BG_COLOR, foreground=TEXT_COLOR, font=self.font_main)
        style.configure("Card.TLabel", background=CARD_COLOR, foreground=TEXT_COLOR, font=self.font_main)
        style.configure("Header.TLabel", background=BG_COLOR, foreground=PRIMARY_COLOR, font=self.font_header)
        style.configure("Sub.TLabel", background=CARD_COLOR, foreground=SUBTEXT_COLOR, font=("Segoe UI", 9))
        
        # Buttons
        style.configure("Primary.TButton", 
                        background=PRIMARY_COLOR, 
                        foreground="white", 
                        font=self.font_bold)
        style.map("Primary.TButton", 
                  background=[("active", PRIMARY_HOVER), ("disabled", BORDER_COLOR)])
        
        style.configure("Secondary.TButton", 
                        background=BORDER_COLOR, 
                        foreground=TEXT_COLOR, 
                        font=self.font_main)
        
        # Entry
        style.configure("TEntry", fieldbackground=BG_COLOR, foreground=TEXT_COLOR, bordercolor=BORDER_COLOR)
        
        # Checkbox
        style.configure("TCheckbutton", background=CARD_COLOR, foreground=TEXT_COLOR, font=self.font_main)

    def create_widgets(self):
        # Header
        header_frame = ttk.Frame(self.root, padding=(0, 30, 0, 10))
        header_frame.pack(fill="x")
        
        ttk.Label(header_frame, text="TraderLogPro", style="Header.TLabel").pack()
        ttk.Label(header_frame, text="SISTEMA DE GESTÃO DE LICENÇAS", style="Sub.TLabel", background=BG_COLOR).pack()

        # Main Card
        main_card = tk.Frame(self.root, bg=CARD_COLOR, bd=1, highlightbackground=BORDER_COLOR, highlightthickness=1)
        main_card.pack(fill="both", expand=True, padx=30, pady=20)
        
        content = ttk.Frame(main_card, padding=20, style="Card.TFrame")
        content.pack(fill="both", expand=True)

        # Plan Selection
        ttk.Label(content, text="PLANO DE ACESSO", style="Card.TLabel").pack(anchor="w")
        self.plan_var = tk.StringVar(value="Pro")
        self.plan_combo = ttk.Combobox(content, textvariable=self.plan_var, state="readonly", font=self.font_main)
        self.plan_combo['values'] = ("Pro", "Trial", "Enterprise", "Lifetime")
        self.plan_combo.pack(fill="x", pady=(5, 20))

        # Duration
        ttk.Label(content, text="DURAÇÃO (DIAS)", style="Card.TLabel").pack(anchor="w")
        self.days_var = tk.StringVar(value="7")
        self.days_entry = tk.Entry(content, textvariable=self.days_var, 
                                   bg=BG_COLOR, fg=TEXT_COLOR, 
                                   insertbackground=TEXT_COLOR,
                                   font=self.font_main, bd=0, 
                                   highlightthickness=1, 
                                   highlightbackground=BORDER_COLOR,
                                   highlightcolor=PRIMARY_COLOR)
        self.days_entry.pack(fill="x", pady=(5, 5), ipady=8)

        # Lifetime Option
        self.lifetime_chk = ttk.Checkbutton(content, text="Ativar Licença Vitalícia", 
                                            variable=self.is_lifetime, 
                                            command=self.on_lifetime_toggle,
                                            style="TCheckbutton")
        self.lifetime_chk.pack(anchor="w", pady=(0, 20))

        # Customer ID (PIN)
        ttk.Label(content, text="CUSTOMER ID (PIN)", style="Card.TLabel").pack(anchor="w")
        self.cid_var = tk.StringVar()
        self.cid_entry = tk.Entry(content, textvariable=self.cid_var, 
                                   bg=BG_COLOR, fg=TEXT_COLOR, 
                                   insertbackground=TEXT_COLOR,
                                   font=self.font_main, bd=0, 
                                   highlightthickness=1, 
                                   highlightbackground=BORDER_COLOR,
                                   highlightcolor=PRIMARY_COLOR)
        self.cid_entry.pack(fill="x", pady=(5, 20), ipady=8)

        # Action Button
        self.gen_btn = ttk.Button(content, text="GERAR CHAVE ASSINADA", 
                                  command=self.generate_key, 
                                  style="Primary.TButton",
                                  cursor="hand2")
        self.gen_btn.pack(fill="x", pady=(10, 20), ipady=10)

        # Result Display
        ttk.Label(content, text="CHAVE DE LICENÇA", style="Card.TLabel").pack(anchor="w")
        self.output_text = tk.Text(content, height=5, bg=BG_COLOR, fg="#4ade80", 
                                   font=self.font_mono, bd=0, padx=10, pady=10,
                                   highlightthickness=1, highlightbackground=BORDER_COLOR,
                                   wrap="char")
        self.output_text.pack(fill="x", pady=(5, 10))
        self.output_text.config(state="disabled")

        # Copy Button
        self.copy_btn = ttk.Button(content, text="COPIAR CHAVE", 
                                   command=self.copy_to_clipboard, 
                                   style="Secondary.TButton",
                                   cursor="hand2")
        self.copy_btn.pack(fill="x", pady=(0, 10))

        # Export Button
        self.export_btn = ttk.Button(content, text="EXPORTAR ARQUIVO .LIC", 
                                     command=self.export_lic, 
                                     style="Primary.TButton",
                                     cursor="hand2",
                                     state="disabled")
        self.export_btn.pack(fill="x")

        # Footer
        footer = ttk.Label(self.root, text="Chave Secreta: TraderLogPro_2026", style="Sub.TLabel", background=BG_COLOR)
        footer.pack(side="bottom", pady=10)

    def on_lifetime_toggle(self): # Renamed from toggle_lifetime
        if self.is_lifetime.get():
            self.days_entry.config(state="disabled", disabledbackground=CARD_COLOR)
            self.plan_var.set("Lifetime")
        else:
            self.days_entry.config(state="normal")
            self.plan_var.set("Pro")

    def generate_key(self):
        try:
            plan = self.plan_var.get()
            cid = self.cid_entry.get().strip().upper() # Get Customer ID
            
            exp_iso = None
            if not self.is_lifetime.get(): # Changed from self.lifetime_var.get()
                try:
                    days = int(self.days_var.get())
                    if days <= 0: raise ValueError
                    exp_date = datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(days=days)
                    exp_iso = exp_date.isoformat().replace("+00:00", "Z") # Original format
                except ValueError:
                    messagebox.showerror("Erro", "Insira um número válido de dias (ex: 7).")
                    return
            
            payload = {
                "plan": plan,
                "exp": exp_iso,
                "cid": cid, # Added Customer ID
                "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z") # Original format
            }
            
            payload_json = json.dumps(payload, separators=(',', ':'))
            payload_b64 = base64.b64encode(payload_json.encode('utf-8')).decode('utf-8')
            data_to_sign = f"{PREFIX}-{VERSION}-{payload_b64}"
            
            signature = hmac.new(
                SECRET_KEY.encode('utf-8'),
                data_to_sign.encode('utf-8'),
                hashlib.sha256
            ).hexdigest()
            
            final_key = f"{data_to_sign}-{signature}"
            
            self.output_text.config(state="normal")
            self.output_text.delete("1.0", tk.END)
            self.output_text.insert("1.0", final_key)
            self.output_text.config(state="disabled")
            self.export_btn.config(state="normal")
            
        except Exception as e:
            messagebox.showerror("Erro Crítico", f"Falha na geração: {str(e)}")

    def copy_to_clipboard(self):
        key = self.output_text.get("1.0", tk.END).strip()
        if key:
            self.root.clipboard_clear()
            self.root.clipboard_append(key)
            self.root.update()
            messagebox.showinfo("Sucesso", "Chave copiada com sucesso!")

    def export_lic(self):
        key = self.output_text.get("1.0", tk.END).strip()
        if not key: return
        
        from tkinter import filedialog
        file_path = filedialog.asksaveasfilename(
            defaultextension=".lic",
            filetypes=[("License Files", "*.lic"), ("All Files", "*.*")],
            initialfile="license.lic"
        )
        
        if file_path:
            try:
                with open(file_path, "w", encoding="utf-8") as f:
                    f.write(key)
                messagebox.showinfo("Sucesso", f"Licença salva em:\n{file_path}")
            except Exception as e:
                messagebox.showerror("Erro", f"Falha ao salvar arquivo: {str(e)}")

if __name__ == "__main__":
    root = tk.Tk()
    # Center Window
    window_width = 520
    window_height = 580
    screen_width = root.winfo_screenwidth()
    screen_height = root.winfo_screenheight()
    center_x = int(screen_width/2 - window_width / 2)
    center_y = int(screen_height/2 - window_height / 2)
    root.geometry(f'{window_width}x{window_height}+{center_x}+{center_y}')
    
    app = ModernLicenseGenerator(root)
    root.mainloop()
