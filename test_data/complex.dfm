object frmComplex: TForm
  Left = 0
  Top = 0
  Width = 1024
  Height = 768
  Caption = 'Complex Form'
  object pnlTop: TPanel
    Left = 0
    Top = 0
    Width = 1024
    Height = 50
    Align = alTop
    Caption = 'Top Panel'
  end
  object pgcMain: TPageControl
    Left = 0
    Top = 50
    Width = 1024
    Height = 718
    Align = alClient
    object tabClientes: TTabSheet
      Caption = 'Clientes'
      object grdClientes: TStringGrid
        Left = 8
        Top = 8
        Width = 1000
        Height = 400
        Anchors = [akLeft, akTop, akRight, akBottom]
      end
    end
    object tabProdutos: TTabSheet
      Caption = 'Produtos'
      object grpFiltro: TGroupBox
        Left = 8
        Top = 8
        Width = 500
        Height = 100
        Caption = 'Filtro'
        object lblBusca: TLabel
          Left = 8
          Top = 24
          Caption = 'Buscar:'
        end
        object edtBusca: TEdit
          Left = 60
          Top = 20
          Width = 430
          Height = 21
          Anchors = [akLeft, akTop, akRight]
        end
        object btnBuscar: TButton
          Left = 416
          Top = 60
          Width = 75
          Height = 25
          Caption = 'Buscar'
          Anchors = [akRight, akBottom]
        end
      end
      object lstProdutos: TListBox
        Left = 8
        Top = 120
        Width = 500
        Height = 300
      end
      object memDescricao: TMemo
        Left = 520
        Top = 8
        Width = 400
        Height = 412
      end
      object imgProduto: TImage
        Left = 520
        Top = 430
        Width = 200
        Height = 200
      end
      object chkAtivo: TCheckBox
        Left = 8
        Top = 430
        Caption = 'Somente Ativos'
      end
      object radTodos: TRadioButton
        Left = 8
        Top = 460
        Caption = 'Todos'
      end
      object cmbCategoria: TComboBox
        Left = 8
        Top = 490
        Width = 200
        Height = 21
      end
      object btnSalvar: TBitBtn
        Left = 420
        Top = 490
        Width = 75
        Height = 25
        Caption = 'Salvar'
      end
    end
  end
end
