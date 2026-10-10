class Profile
  attr_reader :display_name, :naïve
  attr_writer :nickname
  attr_accessor :email, :café
  has_one :avatar
  belongs_to :author
  has_many :comments
  scope :active, -> { where(active: true) }
  delegate :name, :email, to: :author
  delegate :name, to: :author, prefix: true
  delegate :name, to: :author, prefix: false
  delegate :café, to: :écrivain, prefix: true
end
