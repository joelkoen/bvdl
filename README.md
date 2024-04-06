# bvdl

bvdl is a tool for scraping product information from Bazaarvoice.

## Usage

bvdl requires a deployment ID or passkey and will print raw JSON data on individual lines. You'll need to find a deployment ID or passkey by inspecting network requests. bvdl can use a deployment ID to find a passkey for you.

```txt
https://apps.bazaarvoice.com/deployments/[CLIENT_NAME]/[SITE_NAME]/production/[LOCALE]/bv.js
https://display.ugc.bazaarvoice.com/static/[CLIENT_NAME]/[SITE_NAME]/[LOCALE]/bvapi.js
    -> CLIENT_NAME/SITE_NAME/LOCALE

  OR

https://api.bazaarvoice.com/data/[...].json?passkey=[PASSKEY]
   -> PASSKEY
```

For example, by inspecting the requests made on [this page](https://www.lenovo.com/au/en/p/laptops/thinkpad/thinkpadx1/thinkpad-x1-carbon-gen-12-(14-inch-intel)/21kccto1wwau3) I can see network requests to the following URLs:

```
https://apps.bazaarvoice.com/deployments/lenovo-au/main_site/production/en_AU/bv.js
https://api.bazaarvoice.com/data/products.json?filter=id:eq:LEN101T0083&passkey=capxgdWJRBjQt4SmgzkMVZPiinJsxVDEIfrtpsf4CfrEw&apiversion=5.5&...
```

The deployment ID for Lenovo's Australian website is `lenovo-au/main_site/en_AU` and a valid passkey is `capxgdWJRBjQt4SmgzkMVZPiinJsxVDEIfrtpsf4CfrEw`. I can now scrape data from Bazaarvoice using:

```sh
$ bvdl lenovo-au/main_site/en_AU > lenovo-au.jsonl
Found passkey: canlmj0oyOzMW7Ig6GN80LZi42Id1Jeqxo0Go9uysOtzI
Fetching 7335 products...

$ bvdl capxgdWJRBjQt4SmgzkMVZPiinJsxVDEIfrtpsf4CfrEw > lenovo-au.jsonl
Fetching 7335 products...
```

Note that Bazaarvoice provides multiple passkeys for the same deployment, and the above commands will result in the same output despite bvdl finding a different key from the deployment ID.

When scraping a large amount of products, you'll probably want to use pv to see progress:

```sh
$ bvdl lenovo-au/main_site/en_AU | pv -albt > lenovo-au.jsonl 
Found passkey: canlmj0oyOzMW7Ig6GN80LZi42Id1Jeqxo0Go9uysOtzI
Fetching 7335 products...
7.34k 0:00:07 [1.02k/s]
```
