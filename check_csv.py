line = 'BITZ25;10/12/2025 09:39:47;10/12/2025 10:01:07;21min19s;10;10;V;504.920,00;502.780,00;508.540,00;Sim;-214,00;-2.140,00;-214,00;'
fields = line.split(';')
header = 'Ativo;Abertura;Fechamento;Tempo Operacao;Qtd Compra;Qtd Venda;Lado;Preco Compra;Preco Venda;Preco de Mercado;Medio;Res. Intervalo Bruto;Res. Intervalo (%);Res. Operacao;Res. Operacao (%);TET;Total'
h_fields = header.split(';')

for i in range(max(len(fields), len(h_fields))):
    h = h_fields[i] if i < len(h_fields) else '???'
    f = fields[i] if i < len(fields) else '???'
    print(f"{i}: {h} => {f}")
