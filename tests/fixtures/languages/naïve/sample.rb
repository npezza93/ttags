module Demo
  class Café
    VERSION = 1
    def initialize; end
    def goûter; end
    def self.build; end
    alias taste goûter
  end
end
