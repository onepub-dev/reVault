# frozen_string_literal: true

require 'rbconfig'

module Revault
  module NativeLibrary
    module_function

    def path(explicit_path = nil)
      raise ArgumentError, 'native library path must not be empty' if explicit_path == ''
      return @selected_path if defined?(@selected_path) && explicit_path.nil?
      inherited = ENV['REVAULT_LIBRARY']
      selected = explicit_path || (inherited unless inherited.nil? || inherited.empty?) || bundled_path
      selected = resolve_windows_bare_name(selected)
      if defined?(@selected_path) && @selected_path != selected
        raise 'the process-wide reVault native library is already selected'
      end
      @selected_path = selected
    end

    def resolve_windows_bare_name(path)
      return path unless RbConfig::CONFIG['host_os'].match?(/mswin|mingw/)
      return path if path.match?(%r{[/\\]})

      ENV.fetch('PATH', '').split(File::PATH_SEPARATOR).each do |directory|
        candidate = File.expand_path(path, directory)
        return candidate if File.file?(candidate)
      end
      path
    end

    def bundled_path
      bundled = bundled_location
      return bundled if File.file?(bundled)
      raise "revault-api native carrier is missing: #{bundled}; install the matching platform gem"
    end

    def bundled_location
      cpu = case RbConfig::CONFIG['host_cpu']
            when 'x86_64', 'amd64', 'x64' then 'x86_64'
            when 'aarch64', 'arm64' then 'aarch64'
            else raise "unsupported reVault architecture: #{RbConfig::CONFIG['host_cpu']}"
            end
      target, library = case RbConfig::CONFIG['host_os']
                        when /linux/ then ["linux-#{cpu}-gnu", 'librevault_api.so']
                        when /darwin/ then ["macos-#{cpu}", 'librevault_api.dylib']
                        when /mswin|mingw/ then ["windows-#{cpu}-msvc", 'revault_api.dll']
                        else raise "unsupported reVault operating system: #{RbConfig::CONFIG['host_os']}"
                        end
      File.expand_path("../../native/#{target}/#{library}", __dir__)
    end

    # Bind struct-return adapters to the same carrier as ordinary operations.
    def bind_shim
      addresses = Revault::Shim.singleton_methods(false).grep(/^ruby_/).to_h do |name|
        symbol = name.to_s.delete_prefix('ruby_')
        [symbol, Revault::Native.handler[symbol]]
      end
      resolver = Fiddle::Closure::BlockCaller.new(
        Fiddle::TYPE_VOIDP, [Fiddle::TYPE_VOIDP]
      ) { |name| addresses.fetch(Fiddle::Pointer.new(name).to_s) }
      Fiddle::Function.new(
        Revault::Shim.handler['ruby_bind_library'],
        [Fiddle::TYPE_VOIDP], Fiddle::TYPE_VOID
      ).call(resolver)
    end

    def shim_path
      name = case RbConfig::CONFIG['host_os']
             when /linux/ then 'librevault_ruby_shim.so'
             when /darwin/ then 'librevault_ruby_shim.dylib'
             when /mswin|mingw/ then 'revault_ruby_shim.dll'
             else raise "unsupported reVault operating system: #{RbConfig::CONFIG['host_os']}"
             end
      bundled = File.join(File.dirname(bundled_location), name)
      return bundled if File.file?(bundled)
      raise "revault-api Ruby native shim is missing beside #{bundled_location}"
    end
  end
end
