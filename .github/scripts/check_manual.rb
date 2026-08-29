#!/usr/bin/env ruby
# frozen_string_literal: true

require 'open3'
require 'pathname'

ROOT = Pathname.new(__dir__).join('../..').expand_path
MANUAL = ROOT.join('manual')
failures = []

def markdown_links(text)
  text.scan(/\[[^\]]*\]\(([^)]+)\)/).flatten
end

summary = MANUAL.join('SUMMARY.md').read
summary_paths = summary.scan(/\]\(([^)#]+)(?:#[^)]+)?\)/).flatten

summary_paths.group_by(&:itself).each do |path, copies|
  failures << "SUMMARY.md lists #{path} more than once" if copies.length > 1
end

summary_paths.each do |path|
  failures << "SUMMARY.md exposes internal source #{path}" if path.match?(%r{\A(?:agents\.md|docs/|epage_file/|rust/)})
  failures << "SUMMARY.md target does not exist: #{path}" unless MANUAL.join(path).file?
end

published = summary_paths.map { |path| MANUAL.join(path) }.select(&:file?)

published.each do |file|
  text = file.read
  markdown_links(text).each do |link|
    next if link.match?(%r{\A(?:https?://|mailto:)})

    relative = link.sub(/#.*/, '')
    next if relative.empty?

    target = file.dirname.join(relative).cleanpath
    failures << "#{file.relative_path_from(ROOT)} has missing link #{link}" unless target.exist?
  end

  {
    'hint' => ['{% hint ', '{% endhint %}'],
    'tabs' => ['{% tabs %}', '{% endtabs %}'],
    'stepper' => ['{% stepper %}', '{% endstepper %}'],
    'columns' => ['{% columns %}', '{% endcolumns %}']
  }.each do |name, (opening, closing)|
    failures << "#{file.relative_path_from(ROOT)} has unbalanced #{name} blocks" unless text.scan(opening).length == text.scan(closing).length
  end
end

published_text = published.map(&:read).join("\n")
forbidden = {
  'invalid lbox executable alias' => /(?:\A|[`\s])lbox\s/,
  'retired vault identity command' => /vault identity/,
  'retired access add command' => /access add/,
  'retired Auto Open command' => /session auto-open (?:on|off)/,
  'stale lockbox_core API' => /lockbox_core/,
  'stale lockbox_vault API' => /lockbox_vault/,
  'retired lockbox_key_server executable' => /lockbox_key_server/,
  'unfinished TODO marker' => /\bTODO\b/,
  'unfinished HELP required marker' => /HELP required/
}
forbidden.each do |label, pattern|
  failures << "published manual contains #{label}" if published_text.match?(pattern)
end

published.each do |file|
  in_code = false
  file.each_line.with_index(1) do |line, number|
    in_code = !in_code if line.start_with?('```')
    next unless in_code

    if line.match?(/^\s*(?:lbx|lockbox)\s+(?:open|close|add|cat|extract|list|move|remove|rm)\s+\S+\.lbox\b/)
      failures << "#{file.relative_path_from(ROOT)}:#{number} uses old command-before-Lockbox ordering"
    end
  end
end

config_source = ROOT.join('rust/revault_key_server/src/main.rs').read
config_body = config_source[/fn apply_config_value\(.*?\nfn parse_topology_server/m]
if config_body.nil?
  failures << 'could not find key-server apply_config_value implementation'
else
  config_keys = config_body.lines.filter_map do |line|
    next unless line.include?('=>') && line.lstrip.start_with?('"')

    line.scan(/"([a-z][a-z0-9_]*)"/).flatten
  end.flatten.uniq
  config_keys -= %w[default_ttl_seconds max_ttl_seconds]
  config_keys += %w[topology_server route]
  config_page = MANUAL.join('key-sharing-service/configuration.md').read
  config_keys.sort.each do |key|
    failures << "key-server setting #{key} is not documented" unless config_page.include?("`#{key}`") || config_page.include?(key)
  end
end

vars = MANUAL.join('.gitbook/vars.yaml').read
{
  'cli_version' => ROOT.join('rust/revault_cli/Cargo.toml'),
  'key_server_version' => ROOT.join('rust/revault_key_server/Cargo.toml')
}.each do |variable, manifest|
  expected = manifest.read[/^version\s*=\s*"([^"]+)"/, 1]
  actual = vars[/^#{variable}:\s*(\S+)/, 1]
  failures << "#{variable} is #{actual.inspect}; expected #{expected.inspect}" unless actual == expected
end

binding_names = %w[c cpp csharp dart go java javascript kotlin lua php python ruby rust swift typescript wasm]
binding_names.each do |name|
  readme = ROOT.join('bindings', name, 'README.md')
  text = readme.read
  failures << "#{readme.relative_path_from(ROOT)} does not link the manual" unless text.include?('https://docs.revault.onepub.dev/')
  failures << "#{readme.relative_path_from(ROOT)} has no worked code block" if text.scan(/^```/).length < 2
  unless text.match?(/\b(?:close|free|dispose|drop|drops|owned|RAII|AutoCloseable|defer)\b/i)
    failures << "#{readme.relative_path_from(ROOT)} does not state its ownership boundary"
  end
end

if ARGV[0]
  cli = Pathname.new(ARGV[0]).expand_path
  failures << "CLI executable not found: #{cli}" unless cli.executable?
  if cli.executable?
    ROOT.join('.github/scripts/manual_cli_forms.txt').each_line.with_index(1) do |line, number|
      form = line.sub(/#.*/, '').strip
      next if form.empty?

      _stdout, stderr, status = Open3.capture3(cli.to_s, *form.split, '--help')
      failures << "CLI form line #{number} is not accepted: #{form}: #{stderr.strip}" unless status.success?
    end
  end
end

if failures.empty?
  puts "Manual validation passed (#{published.length} published pages)."
  exit 0
end

warn failures.map { |failure| "- #{failure}" }.join("\n")
exit 1
