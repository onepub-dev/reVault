# Lockbox management

Moving a lockbox:

You can move a lockbox at anytime without consequences, however if you use `lbx open` or `lbx session default ...` you will get a cleaner result if you tell reVault that you are moving the lockbox.

If your lockbox is called \`system\_api\_keys.lbox\`

`lbx vault lockbox move system_api_keys.lbox /some/new/path/system_api_keys.lbox`

or&#x20;

`lbx vault lockbox move system_api_keys.lbox /some/new/directory`



Manually:

If you are going to move the lockbox manually then you should also move the lock file

mv system\_api\_keys.lbox ...

mv .system\_api\_keys.lbox.lock ...

You should also tell reVault to forget the lockbox:

lbx session forget&#x20;



