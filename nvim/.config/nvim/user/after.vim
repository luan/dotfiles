" Called after everything just before setting a default colorscheme
" Configure you own bindings or other preferences. e.g.:

" set nonumber " No line numbers

" let g:gitgutter_signs = 0 " No git gutter signs
" augroup config#after
"   autocmd!
"   autocmd VimEnter * GitGutterDisable
" augroup end
" let g:SignatureEnabledAtStartup = 0 " Do not show marks

" let g:ale_open_list = 1

let g:tokyonight_italic_functions = v:true
let g:tokyonight_sidebars = [ "quickfix", "__vista__", "terminal" ]

if filereadable($XDG_RUNTIME_DIR . '/lighttheme')
  set background=light
  let g:lightline.colorscheme = 'solarized'
  colorscheme base16-solarized-light
else
  set background=dark
  " let g:tokyonight_style = "night"
  let g:lightline.colorscheme = 'material'
  colorscheme tokyonight
endif


au BufNewFile,BufRead /*.rasi setf css

if filereadable(expand('<sfile>:h') . '/post-after.vim')
  runtime user/post-after.vim
endif

autocmd FileType pyst setlocal commentstring=#\ %s

" Experimental stuff below
if !exists('g:luan_experimental')
  finish
endif

lua <<EOF
require'nvim-treesitter.configs'.setup {
  ensure_installed = "maintained",     -- one of "all", "language", or a list of languages
  highlight = {
    enable = true,              -- false will disable the whole extension
  },
  incremental_selection = {
    enable = true,
    keymaps = {
      init_selection = "gnn",
      node_incremental = "grn",
      scope_incremental = "grc",
      node_decremental = "grm",
    },
  },
  refactor = {
    highlight_definitions = { enable = true },
    -- highlight_current_scope = { enable = true },
    smart_rename = {
      enable = true,
      keymaps = {
        smart_rename = "grr",
      },
    },
    navigation = {
      enable = true,
      keymaps = {
        goto_definition = "gnd",
        list_definitions = "gnD",
        list_definitions_toc = "gO",
        goto_next_usage = "<a-*>",
        goto_previous_usage = "<a-#>",
      },
    },
  },
  textobjects = {
    select = {
      enable = true,
      keymaps = {
        -- You can use the capture groups defined in textobjects.scm
        ["af"] = "@function.outer",
        ["if"] = "@function.inner",
        ["ac"] = "@class.outer",
        ["ic"] = "@class.inner",
      },
    },
  },
  playground = {
    enable = true,
    disable = {},
    updatetime = 25, -- Debounced time for highlighting nodes in the playground from source code
    persist_queries = false -- Whether the query persists across vim sessions
  },
}

require('bufferline').setup{}

require('gitsigns').setup()

require('lualine').setup{
  options = { theme = 'tokyonight' },
  extensions = { 'fzf' },
  sections = {
    lualine_a = {'mode'},
    lualine_b = {
      'branch',
      {
        'diagnostics',
        sources = {'ale', 'coc'},
      },
    },
    lualine_c = {'filename'},
    lualine_x = {
      'encoding',
      'fileformat',
      'filetype',
    },
    lualine_y = {'progress'},
    lualine_z = {'location'}
  },
  inactive_sections = {
    lualine_a = {},
    lualine_b = {},
    lualine_c = {'filename'},
    lualine_x = {'location'},
    lualine_y = {},
    lualine_z = {},
  },
}

local saga = require 'lspsaga'

-- add your config value here
-- default value
-- use_saga_diagnostic_sign = true
-- error_sign = '',
-- warn_sign = '',
-- hint_sign = '',
-- infor_sign = '',
-- dianostic_header_icon = '   ',
-- code_action_icon = ' ',
-- code_action_prompt = {
--   enable = true,
--   sign = true,
--   sign_priority = 20,
--   virtual_text = true,
-- },
-- finder_definition_icon = '  ',
-- finder_reference_icon = '  ',
-- max_preview_lines = 10, -- preview lines of lsp_finder and definition preview
-- finder_action_keys = {
--   open = 'o', vsplit = 's',split = 'i',quit = 'q',scroll_down = '<C-f>', scroll_up = '<C-b>' -- quit can be a table
-- },
-- code_action_keys = {
--   quit = 'q',exec = '<CR>'
-- },
-- rename_action_keys = {
--   quit = '<C-c>',exec = '<CR>'  -- quit can be a table
-- },
-- definition_preview_icon = '  '
-- "single" "double" "round" "plus"
-- border_style = "single"
-- rename_prompt_prefix = '➤',
-- if you don't use nvim-lspconfig you must pass your server name and
-- the related filetypes into this table
-- like server_filetype_map = {metals = {'sbt', 'scala'}}
-- server_filetype_map = {}

saga.init_lsp_saga()
EOF

autocmd FileType go setlocal foldmethod=expr foldexpr=nvim_treesitter#foldexpr()
autocmd FileType ruby setlocal foldmethod=expr foldexpr=nvim_treesitter#foldexpr()
set foldmethod=expr
set foldexpr=nvim_treesitter#foldexpr()
set report=2

nnoremap <silent>[b :BufferLineCycleNext<CR>
nnoremap <silent>]b :BufferLineCyclePrev<CR>

let g:indent_blankline_use_treesitter = v:true

set t_ZH=^[[3m
set t_ZR=^[[23m
