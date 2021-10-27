I'm using [telescope.nvim](https://github.com/nvim-telescope/) over sshfs,
which can be *very* laggy (15 seconds for a smallish project for calling 'rg --files' isn't unusual),
and telescope fails when data comes in chunks.

This caches the rg output (tuned to my list of filetypes I want to see) in
~/.cache/ff_filelister/ refreshing it (in the background, deliver old results
first and let telescope search them!) if the cache file is older than timeout
seconds.

Call with ff_listener <timeout> <path>.

Here's how I use it from init.lua


```lua
my_files = function(opts)
    local cmd = {"ff_filelister"}
    table.insert(cmd, "300") -- timeout to refresh
    table.insert(cmd, vim.loop.cwd())
    pickers.new(
        opts,
        {
            prompt_title = "Files",
            finder = finders.new_oneshot_job(vim.tbl_flatten(cmd), opts),
            sorter = conf.generic_sorter(opts)
        }
    ):find()
end

my_files_relative = function(opts)
    local cmd = {"ff_filelister"}
    table.insert(cmd, "300")
	local dir = vim.fn.expand('%:p:h')
    table.insert(cmd, dir)
	if not opts then
		opts = {}
	end
	opts['entry_maker'] = function(entry)
        return {
          value = dir .. '/' .. entry,
          display = entry,
          ordinal = entry,
        }
	end
    pickers.new(
        opts,
        {
            prompt_title = vim.fn.expand('%:p:h'),
            finder = finders.new_oneshot_job(vim.tbl_flatten(cmd), opts),
            sorter = conf.generic_sorter(opts)
        }
    ):find()
end


vim.api.nvim_exec([[
noremap <leader>y :Telescope neoclip<CR>
noremap <leader>f :lua my_files()<CR>
noremap <leader>F :lua my_files_relative()<CR>
]], false)

```
