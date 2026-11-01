FROM ruby:3.3-bookworm
ARG REVAULT_GEM
COPY artifacts/ruby/${REVAULT_GEM} /tmp/revault.gem
RUN gem install /tmp/revault.gem --no-document
COPY bindings/e2e/ruby/conformance.rb /tmp/conformance.rb
ENV REVAULT_E2E_LOADER_SMOKE=frozen-bookworm-payload
CMD ["ruby", "/tmp/conformance.rb"]
