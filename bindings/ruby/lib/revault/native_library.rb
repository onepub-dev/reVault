# frozen_string_literal: true

require 'rbconfig'

module Revault
  module NativeLibrary
    module_function

    def path
      return ENV['REVAULT_LIBRARY'] unless ENV['REVAULT_LIBRARY'].to_s.empty?
      cpu = case RbConfig::CONFIG['host_cpu']
            when 'x86_64', 'amd64' then 'x86_64'
            when 'aarch64', 'arm64' then 'aarch64'
            else raise "unsupported reVault architecture: #{RbConfig::CONFIG['host_cpu']}"
            end
      target, library = case RbConfig::CONFIG['host_os']
                        when /linux/ then ["linux-#{cpu}-gnu", 'librevault_api.so']
                        when /darwin/ then ["macos-#{cpu}", 'librevault_api.dylib']
                        when /mswin|mingw/ then ["windows-#{cpu}-msvc", 'revault_api.dll']
                        else raise "unsupported reVault operating system: #{RbConfig::CONFIG['host_os']}"
                        end
      bundled = File.expand_path("../../native/#{target}/#{library}", __dir__)
      return bundled if File.file?(bundled)
      raise "revault-api native carrier is missing for #{target}; set REVAULT_LIBRARY for development"
    end

    def shim_path
      return ENV['REVAULT_RUBY_SHIM'] unless ENV['REVAULT_RUBY_SHIM'].to_s.empty?
      name = case RbConfig::CONFIG['host_os']
             when /linux/ then 'librevault_ruby_shim.so'
             when /darwin/ then 'librevault_ruby_shim.dylib'
             when /mswin|mingw/ then 'revault_ruby_shim.dll'
             else raise "unsupported reVault operating system: #{RbConfig::CONFIG['host_os']}"
             end
      bundled = File.join(File.dirname(path), name)
      return bundled if File.file?(bundled)
      raise "revault-api Ruby native shim is missing beside #{path}"
    end
  end
end
