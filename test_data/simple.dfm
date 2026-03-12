object frmCadastro: TForm
  Left = 100
  Top = 100
  Width = 800
  Height = 600
  Caption = 'Cadastro de Clientes'
  object pnlTopo: TPanel
    Left = 0
    Top = 0
    Width = 800
    Height = 50
    Align = alTop
    object lblTitulo: TLabel
      Left = 8
      Top = 16
      Caption = 'Clientes'
      Anchors = [akLeft, akTop]
    end
  end
  object pnlCliente: TPanel
    Left = 0
    Top = 50
    Width = 800
    Height = 550
    Align = alClient
    object lblNome: TLabel
      Left = 16
      Top = 16
      Caption = 'Nome:'
    end
    object edtNome: TEdit
      Left = 16
      Top = 32
      Width = 760
      Height = 21
      Anchors = [akLeft, akTop, akRight]
    end
    object lblEmail: TLabel
      Left = 16
      Top = 64
      Caption = 'Email:'
    end
    object edtEmail: TEdit
      Left = 16
      Top = 80
      Width = 760
      Height = 21
      Anchors = [akLeft, akTop, akRight]
    end
    object btnSalvar: TButton
      Left = 700
      Top = 520
      Width = 75
      Height = 25
      Caption = 'Salvar'
      Anchors = [akRight, akBottom]
    end
  end
end
