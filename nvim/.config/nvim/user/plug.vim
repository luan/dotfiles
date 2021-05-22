" Add your own plugins
" Plug 'scrooloose/nerdtree', { 'on':  'NERDTreeToggle' }
" Plug '~/my-prototype-plugin'
" ...

Plug '~/src/misc/cugini/vim-vortex2'
Plug '~/src/misc/cugini/vim-pyst'

" Experimental stuff below
if !exists('g:luan_experimental')
  finish
endif

Plug 'nvim-treesitter/nvim-treesitter'
Plug 'nvim-treesitter/nvim-treesitter-refactor'
Plug 'nvim-treesitter/nvim-treesitter-textobjects'
Plug 'romgrk/nvim-treesitter-context'
Plug 'nvim-treesitter/playground'

Plug 'nvim-lua/popup.nvim'
Plug 'nvim-lua/plenary.nvim'
Plug 'nvim-lua/telescope.nvim'

Plug 'nvim-telescope/telescope-fzy-native.nvim'

Plug 'kyazdani42/nvim-web-devicons'
Plug 'akinsho/nvim-bufferline.lua'

" Plug 'kyazdani42/nvim-tree.lua'

Plug 'lukas-reineke/indent-blankline.nvim', { 'branch': 'lua' }

Plug 'lewis6991/gitsigns.nvim'

Plug 'hoob3rt/lualine.nvim'

Plug 'folke/tokyonight.nvim'

Plug 'neovim/nvim-lspconfig'
Plug 'glepnir/lspsaga.nvim'
