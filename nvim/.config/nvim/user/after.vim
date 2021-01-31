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

if filereadable($XDG_RUNTIME_DIR . '/lighttheme')
  set background=light
  let g:lightline.colorscheme = 'solarized'
  colorscheme base16-solarized-light
else
  set background=dark
  let g:lightline.colorscheme = 'material'
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

" telescope
lua <<EOF
require('telescope').setup{
  defaults = {
    vimgrep_arguments = {
      'rg',
      '--color=never',
      '--no-heading',
      '--with-filename',
      '--line-number',
      '--column',
      '--smart-case'
    },
    prompt_position = "bottom",
    prompt_prefix = ">",
    selection_strategy = "reset",
    sorting_strategy = "descending",
    layout_strategy = "horizontal",
    layout_defaults = {
      -- TODO add builtin options.
    },
    file_ignore_patterns = {},
    shorten_path = true,
    winblend = 0,
    width = 0.75,
    preview_cutoff = 120,
    results_height = 1,
    results_width = 0.8,
    border = {},
    borderchars = { '─', '│', '─', '│', '╭', '╮', '╯', '╰'},
    color_devicons = true,
    use_less = true,
    set_env = { ['COLORTERM'] = 'truecolor' }, -- default { }, currently unsupported for shells like cmd.exe / powershell.exe
  }
}
require('telescope').load_extension('fzy_native')
EOF

" treesitter
lua <<EOF
require'nvim-treesitter.configs'.setup {
  ensure_installed = "all",     -- one of "all", "language", or a list of languages
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
EOF

autocmd FileType go setlocal foldmethod=expr foldexpr=nvim_treesitter#foldexpr()
autocmd FileType ruby setlocal foldmethod=expr foldexpr=nvim_treesitter#foldexpr()
set foldmethod=expr
set foldexpr=nvim_treesitter#foldexpr()
set report=2
