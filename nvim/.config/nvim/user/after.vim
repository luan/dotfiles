" Called after everything just before setting a default colorscheme
" Configure you own bindings or other preferences. e.g.:

set nonumber " No line numbers
" let g:gitgutter_signs = 0 " No git gutter signs
augroup config#after
  autocmd!
  autocmd VimEnter * GitGutterDisable
augroup end
let g:SignatureEnabledAtStartup = 0 " Do not show marks

function! SaveIfUnsaved()
  if &modified
    :silent! w
  endif
endfunction
au CursorHold * :call SaveIfUnsaved()
" Read the file on focus/buffer enter
au FocusGained,BufEnter * :silent! !

