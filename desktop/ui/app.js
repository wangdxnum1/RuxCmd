(function () {
  'use strict';

  var tauriGlobal = window.__TAURI__;
  var backend = tauriGlobal && tauriGlobal.core && typeof tauriGlobal.core.invoke === 'function'
    ? tauriGlobal.core
    : null;
  var hasBackend = backend !== null;
  var previewTag = hasBackend ? '' : '（示例数据）';

  var state = {
    includePrerelease: false,
    checking: false,
    verifying: false,
    install: null,
    release: null,
    prepared: null,
    installing: false,
    activeUpdateId: null,
    handedOff: false
  };

  var PREVIEW_MANAGER = {
    version: '0.5.11',
    target: 'x86_64-pc-windows-msvc',
    buildTime: '2026-11-08T00:00:00Z',
    tauriVersion: '2.x',
    signed: false,
    readOnly: true
  };

  var PREVIEW_INSTALL = {
    method: 'inno',
    suiteVersion: '0.5.11',
    installDir: 'C:\\Users\\<you>\\AppData\\Local\\RuxCmd\\bin',
    sourceCommit: 'a8fb923c6c7725a3829cb1b9f0a572d081a49aaa',
    target: 'x86_64-pc-windows-msvc',
    rustcVersion: '1.99.0',
    binaryCount: 142,
    shimCount: 16,
    countsSource: 'manifest',
    checksumStatus: 'notChecked',
    pathOnUserPath: true,
    packageSha256: null,
    installedAt: '20261109',
    installedAtKind: 'date',
    conflicts: [],
    warnings: []
  };

  var PREVIEW_VERIFICATION = {
    status: 'verified',
    checked: 158,
    missing: [],
    mismatched: [],
    source: 'share/SHA256SUMS',
    message: null
  };

  var PREVIEW_RELEASE = {
    tag: 'v0.5.11',
    version: '0.5.11',
    name: 'RuxCmd v0.5.11',
    prerelease: false,
    draft: false,
    publishedAt: '2026-11-07T15:21:54Z',
    htmlUrl: 'https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.11',
    body: '（预览示例）发布说明将在此显示，真实内容来自发布记录。',
    assets: [
      { name: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip', size: 56136068, digest: 'sha256:e90ab18900000000000000000000000000000000000000000000000042226c7045', state: 'uploaded' },
      { name: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip.sha256', size: 108, digest: 'sha256:94195e19000000000000000000000000000000000000000000000000075df407e', state: 'uploaded' },
      { name: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe', size: 16732287, digest: 'sha256:a7b30d640000000000000000000000000000000000000000000000001d7965eeb17', state: 'uploaded' },
      { name: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe.sha256', size: 114, digest: 'sha256:e6a2a9c800000000000000000000000000000000000000000000000000bd5423', state: 'uploaded' }
    ]
  };

  function el(id) {
    return document.getElementById(id);
  }

  function setText(id, value) {
    var node = el(id);
    if (!node) return;
    node.textContent = value === null || value === undefined || value === '' ? '—' : String(value);
  }

  function setPillIn(node, text, variant) {
    if (!node) return;
    node.textContent = '';
    var pill = document.createElement('span');
    pill.className = 'pill ' + (variant || '');
    var dot = document.createElement('span');
    dot.className = 'dot';
    pill.appendChild(dot);
    pill.appendChild(document.createTextNode(' ' + text));
    node.appendChild(pill);
  }

  function clearNode(node) {
    if (node) node.textContent = '';
  }

  function errorText(error) {
    if (!error) return '未知错误';
    if (typeof error === 'string') return error;
    return error.message || JSON.stringify(error);
  }

  // ---------- navigation ----------

  function setView(name) {
    document.querySelectorAll('.nav-item').forEach(function (item) {
      item.classList.toggle('active', item.getAttribute('data-view') === name);
    });
    document.querySelectorAll('.view').forEach(function (view) {
      view.classList.toggle('active', view.id === 'view-' + name);
    });
  }

  // ---------- about ----------

  function methodLabel(method) {
    switch (method) {
      case 'script': return '脚本安装 (install.ps1)';
      case 'inno': return 'Inno Setup 安装包';
      case 'portable': return '便携解压 / ZIP';
      case 'notInstalled': return '未安装';
      default: return '无法确定';
    }
  }

  function countsSourceLabel(source) {
    if (!source) return '—';
    return source
      .replace('state-file', '安装状态文件')
      .replace('manifest', '发布清单')
      .replace('directory-scan', '目录扫描');
  }

  function shortenCommit(commit) {
    if (!commit) return '—';
    if (commit.length <= 20) return commit;
    return commit.slice(0, 12) + '…' + commit.slice(-5);
  }

  // The script installer records a full UTC timestamp; Inno only records a
  // YYYYMMDD day in its uninstall key. Render each according to its precision
  // instead of pretending the registry day is a timestamp.
  function formatInstalledAt(value, kind) {
    if (!value) return '—';
    if (kind === 'date' && /^\d{8}$/.test(value)) {
      return value.slice(0, 4) + '-' + value.slice(4, 6) + '-' + value.slice(6, 8) + '（安装日期）';
    }
    if (kind === 'timestamp') {
      var parsed = new Date(value);
      if (!isNaN(parsed.getTime())) {
        var pad = function (n) { return String(n).padStart(2, '0'); };
        return parsed.getFullYear() + '-' + pad(parsed.getMonth() + 1) + '-' + pad(parsed.getDate())
          + ' ' + pad(parsed.getHours()) + ':' + pad(parsed.getMinutes()) + '（本地时间）';
      }
    }
    return value;
  }

  // Find the ZIP hash we can show for "发布包 ZIP SHA256". Sources, in order:
  //   1. the script installer's local state file (real, verified at install),
  //   2. the GitHub release ZIP asset digest (after 检查更新),
  //   3. the sidecar cross-check result (same digest, always present after 检查更新).
  // Browser-preview digests are illustrative and are never shown as real.
  function releaseZipSha256(result) {
    if (!hasBackend || !result) return null;
    var normalize = function (value) {
      if (!value) return null;
      var digest = String(value).replace(/^sha256:/, '');
      return /^[0-9a-f]{64}$/i.test(digest) ? digest : null;
    };
    var assets = result.latest && result.latest.assets;
    if (assets) {
      var asset = assets.find(function (item) {
        return item.name && item.name.endsWith('.zip') && item.digest;
      });
      if (asset) {
        var fromAsset = normalize(asset.digest);
        if (fromAsset) return fromAsset;
      }
    }
    var checks = result.sidecarChecks || [];
    for (var i = 0; i < checks.length; i++) {
      var check = checks[i];
      if (check.binary && check.binary.endsWith('.zip') && check.matches === true) {
        var fromCheck = normalize(check.digestSha256 || check.sidecarSha256);
        if (fromCheck) return fromCheck;
      }
    }
    return null;
  }

  function renderPackageSha() {
    var node = el('packageSha');
    if (!node) return;
    var local = state.install && state.install.packageSha256;
    var published = releaseZipSha256(state.release);
    var hash = local || published;
    var copyBtn = el('btnCopySha');
    if (!hash) {
      node.textContent = '—';
      node.removeAttribute('title');
      if (copyBtn) copyBtn.hidden = true;
      setText('packageShaSource', '');
      if (!hasBackend) {
        setText('packageShaHint', '浏览器预览模式：真实运行时显示已发布 ZIP 的 SHA256。');
      } else {
        setText('packageShaHint', '本地未记录整包哈希；切到「检查更新」页点一次按钮后，这里会显示该版本的 ZIP SHA256。');
      }
      return;
    }
    node.textContent = hash;
    node.title = hash;
    if (copyBtn) copyBtn.hidden = false;
    var source = local ? '本地安装状态' : '官方发布记录';
    setText('packageShaSource', '· ' + source);
    setText('packageShaHint', '这是发布 ZIP 的整包 SHA256，与逐文件校验（share/SHA256SUMS）不是同一个东西。');
  }

  function renderManager(manager) {
    if (!manager) return;
    setText('sideManager', manager.version);
    setText('sideTarget', manager.target);
    setText('managerVersion', manager.version + ' · ' + manager.buildTime + ' · tauri ' + manager.tauriVersion);
    var signed = el('signedState');
    if (signed) {
      clearNode(signed);
      setPillIn(signed, manager.signed ? '已签名' : '未签名', manager.signed ? 'ok' : 'warn');
    }
  }

  function renderInstall(info) {
    if (!info) return;
    setText('sideSuite', info.suiteVersion);
    setText('suiteVersion', info.suiteVersion || '未知');
    setText('installDir', info.installDir);
    setText('shimCount', info.shimCount === null || info.shimCount === undefined ? null : info.shimCount + ' 个');
    setText('binaryCount', info.binaryCount === null || info.binaryCount === undefined ? null : info.binaryCount + ' 个');
    setText('countsSource', countsSourceLabel(info.countsSource));
    setText('installTarget', info.target);
    setText('rustcVersion', info.rustcVersion);
    setText('sourceCommit', shortenCommit(info.sourceCommit));
    setText('installedAt', formatInstalledAt(info.installedAt, info.installedAtKind));

    var installPill = el('installPill');
    if (installPill) {
      var label = info.method === 'notInstalled'
        ? '未安装'
        : (info.method === 'unknown' ? '安装状态未知' : '已安装');
      var variant = info.method === 'notInstalled' || info.method === 'unknown' ? 'warn' : 'ok';
      setPillIn(installPill, label, variant);
    }

    var pathNode = el('pathStatus');
    if (pathNode) {
      clearNode(pathNode);
      if (info.pathOnUserPath === true) {
        setPillIn(pathNode, 'PATH 已加入', 'ok');
      } else if (info.pathOnUserPath === false) {
        setPillIn(pathNode, 'PATH 未加入', 'warn');
      } else {
        pathNode.textContent = 'PATH 未知';
      }
    }

    var checksumNode = el('checksumState');
    if (checksumNode) {
      clearNode(checksumNode);
      checksumNode.textContent = '未校验';
    }

    renderPackageSha();
    renderIssues('conflictCard', 'conflictList', info.conflicts);
    renderIssues('warningCard', 'warningList', info.warnings);
  }

  function renderIssues(cardId, listId, items) {
    var card = el(cardId);
    var list = el(listId);
    if (!card || !list) return;
    clearNode(list);
    if (!items || items.length === 0) {
      card.hidden = true;
      return;
    }
    items.forEach(function (text) {
      var li = document.createElement('li');
      li.textContent = text;
      list.appendChild(li);
    });
    card.hidden = false;
  }

  function summarize(items) {
    if (!items || items.length === 0) return '';
    var head = items.slice(0, 3).join('、');
    return items.length > 3 ? head + ' 等 ' + items.length + ' 项' : head;
  }

  function renderVerification(report) {
    var variants = {
      verified: ['通过', 'ok'],
      mismatch: ['不匹配', 'err'],
      missing: ['缺失文件', 'warn'],
      notChecked: ['未校验', 'info']
    };
    var pair = variants[report.status] || ['未校验', 'info'];
    var node = el('checksumState');
    if (node) {
      clearNode(node);
      setPillIn(node, pair[0], pair[1]);
    }
    var detail = report.message;
    if (!detail) {
      detail = report.checked + ' 个文件通过 · 来源 ' + (report.source || 'share/SHA256SUMS');
      if (report.mismatched && report.mismatched.length) {
        detail += ' · 不匹配：' + summarize(report.mismatched);
      }
      if (report.missing && report.missing.length) {
        detail += ' · 缺失：' + summarize(report.missing);
      }
    }
    setText('checksumDetail', detail);
  }

  async function loadVerification() {
    if (state.verifying) return;
    state.verifying = true;
    var button = el('btnVerify');
    if (button) {
      button.disabled = true;
      button.textContent = '校验中…';
    }
    try {
      var report = hasBackend
        ? await backend.invoke('verify_installation')
        : PREVIEW_VERIFICATION;
      renderVerification(report);
    } catch (error) {
      var node = el('checksumState');
      if (node) {
        clearNode(node);
        setPillIn(node, '校验失败', 'err');
      }
      setText('checksumDetail', errorText(error));
    } finally {
      state.verifying = false;
      if (button) {
        button.disabled = false;
        button.textContent = '校验安装文件';
      }
    }
  }

  async function loadManagerAndInstall() {
    var manager = PREVIEW_MANAGER;
    var install = PREVIEW_INSTALL;
    if (hasBackend) {
      try {
        manager = await backend.invoke('get_manager_info');
      } catch (error) {
        toast('读取 Manager 信息失败：' + errorText(error));
      }
      try {
        install = await backend.invoke('get_install_info');
      } catch (error) {
        toast('读取安装信息失败：' + errorText(error));
      }
    }
    renderManager(manager);
    renderInstall(install);
    state.install = install;
    await loadVerification();
  }

  // ---------- update ----------

  function formatBytes(bytes) {
    if (typeof bytes !== 'number' || !isFinite(bytes)) return '—';
    if (bytes >= 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    if (bytes >= 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return bytes + ' B';
  }

  function shortenDigest(digest) {
    if (!digest) return '—';
    var value = digest.replace(/^sha256:/, '');
    if (value.length <= 20) return digest;
    return 'sha256:' + value.slice(0, 12) + '…' + value.slice(-6);
  }

  function renderBanner(variant, title, message, extra) {
    var host = el('updateBanner');
    if (!host) return;
    clearNode(host);

    var banner = document.createElement('div');
    banner.className = 'banner ' + variant;

    var icon = document.createElement('div');
    icon.className = 'ic';
    icon.textContent = { info: 'i', ok: '✓', warn: '!', err: '×' }[variant] || 'i';

    var body = document.createElement('div');
    body.className = 'banner-body';

    var heading = document.createElement('h3');
    heading.textContent = title;
    body.appendChild(heading);

    if (message) {
      var paragraph = document.createElement('p');
      paragraph.textContent = message;
      body.appendChild(paragraph);
    }
    if (extra) {
      body.appendChild(extra);
    }

    var spacer = document.createElement('div');
    spacer.className = 'spacer';

    banner.appendChild(icon);
    banner.appendChild(body);
    banner.appendChild(spacer);
    host.appendChild(banner);
  }

  function buildVersionCompare(currentVersion, latestVersion) {
    var wrap = document.createElement('div');
    wrap.className = 'ver-compare';

    var current = document.createElement('div');
    current.className = 'box';
    current.textContent = currentVersion ? '当前 v' + currentVersion : '当前 未安装';

    var arrow = document.createElement('div');
    arrow.className = 'arrow';
    arrow.textContent = '→';

    var latest = document.createElement('div');
    latest.className = 'box new';
    latest.textContent = '最新 v' + latestVersion;

    wrap.appendChild(current);
    wrap.appendChild(arrow);
    wrap.appendChild(latest);
    return wrap;
  }

  function setBadge(visible) {
    var badge = el('navBadge');
    if (badge) badge.hidden = !visible;
  }

  function clearUpdateDetails() {
    var assetCard = el('assetCard');
    var notesCard = el('notesCard');
    if (assetCard) assetCard.hidden = true;
    if (notesCard) notesCard.hidden = true;
  }

  function renderAssets(result) {
    var latest = result.latest;
    var card = el('assetCard');
    if (!card) return;
    card.hidden = false;
    setText('assetVersion', 'v' + latest.version);

    var body = el('assetBody');
    clearNode(body);
    latest.assets.forEach(function (asset) {
      var row = document.createElement('tr');

      var nameCell = document.createElement('td');
      nameCell.className = 'name';
      nameCell.textContent = asset.name;

      var sizeCell = document.createElement('td');
      sizeCell.className = 'size';
      sizeCell.textContent = formatBytes(asset.size);

      var stateCell = document.createElement('td');
      var tag = document.createElement('span');
      tag.className = 'tag' + (asset.state === 'uploaded' ? '' : ' missing');
      tag.textContent = asset.state === 'uploaded' ? '已上传' : asset.state;
      stateCell.appendChild(tag);

      var digestCell = document.createElement('td');
      var hash = document.createElement('span');
      hash.className = 'hash';
      hash.textContent = shortenDigest(asset.digest);
      digestCell.appendChild(hash);

      row.appendChild(nameCell);
      row.appendChild(sizeCell);
      row.appendChild(stateCell);
      row.appendChild(digestCell);
      if (body) body.appendChild(row);
    });

    setText(
      'assetNote',
      result.assetsComplete
        ? '四个发布基线资产齐全且 state=uploaded。'
        : '缺少或未上传的资产：' + result.missingAssets.join('、')
    );

    var releaseButton = el('btnReleasePage');
    // ReleaseInfo is serialized with serde(rename_all = "camelCase"), so the
    // GitHub html_url field arrives as htmlUrl. Reading html_url here sets the
    // attribute to the string "undefined" and breaks the release-page button.
    if (releaseButton) releaseButton.setAttribute('data-url', latest.htmlUrl || '');
    renderSidecarChecks(result);
  }

  function renderNotes(latest) {
    var card = el('notesCard');
    var body = el('notesBody');
    if (!card || !body) return;
    if (!latest.body) {
      card.hidden = true;
      return;
    }
    body.textContent = latest.body;
    card.hidden = false;
  }

  function renderCacheHint(result) {
    setText(
      'cacheHint',
      result.fromCache
        ? '本次结果来自 ' + result.checkedAt + ' 的检查，' + result.nextRefreshSeconds + ' 秒后可再次刷新。'
        : '已更新于 ' + result.checkedAt + '。'
    );
  }

  function digestOf(value) {
    return value ? shortenDigest('sha256:' + value) : '—';
  }

  function renderSidecarChecks(result) {
    var node = el('sidecarNote');
    if (!node) return;
    var checks = result.sidecarChecks || [];
    if (!checks.length) {
      node.textContent = 'digest ↔ .sha256 交叉校验：未执行。';
      return;
    }
    var passed = checks.filter(function (check) { return check.matches === true; }).length;
    var parts = checks.map(function (check) {
      if (check.matches === true) {
        return check.binary + ' 与 ' + check.sidecar + ' 一致 (' + digestOf(check.digestSha256) + ')';
      }
      if (check.matches === false) {
        return check.binary + ' 与 ' + check.sidecar + ' 不一致（官方记录 '
          + digestOf(check.digestSha256) + ' ↔ sidecar ' + digestOf(check.sidecarSha256) + '）';
      }
      return check.binary + ' 未校验：' + (check.error || '原因未知');
    });
    node.textContent = 'digest ↔ .sha256 交叉校验：' + passed + '/' + checks.length + ' 一致。'
      + parts.join('；') + '。';
  }

  function renderUpdateResult(result) {
    var latest = result.latest;
    if (!latest) {
      clearUpdateDetails();
      setBadge(false);
      renderBanner('err', '没有可用的发布信息' + previewTag, '返回的内容缺少发布对象。');
      return;
    }

    var prerelease = latest.prerelease ? '（预发布）' : '';
    var checks = result.sidecarChecks || [];
    var mismatched = checks.filter(function (check) { return check.matches === false; });
    var unverified = checks.filter(function (check) { return check.matches !== true; });

    if (mismatched.length) {
      setBadge(true);
      renderBanner(
        'err',
        'v' + latest.version + ' 资产哈希不一致' + prerelease + previewTag,
        'digest 与 .sha256 sidecar 交叉校验未通过：'
          + mismatched.map(function (check) { return check.binary; }).join('、')
          + '。此时无法确认版本是否可信，也不判断版本新旧。'
      );
    } else if (!result.assetsComplete) {
      setBadge(true);
      renderBanner(
        'err',
        '发现新版本 v' + latest.version + prerelease + previewTag + '，但资产不完整',
        '缺少资产：' + result.missingAssets.join('、')
          + '。发布基线要求四个资产齐全；此时不判断版本新旧。'
      );
    } else if (checks.length === 0 || unverified.length) {
      setBadge(true);
      renderBanner(
        'warn',
        'v' + latest.version + ' 的资产校验未完成' + prerelease + previewTag,
        (checks.length === 0
          ? '后端没有返回 sidecar 校验结果。'
          : '未完成：' + unverified.map(function (check) {
            return check.binary + '（' + (check.error || '原因未知') + '）';
          }).join('、') + '。')
          + '此时不判断版本新旧。'
      );
    } else if (result.status === 'upToDate') {
      setBadge(false);
      renderBanner(
        'ok',
        '已是最新版本 v' + latest.version + prerelease + previewTag,
        '当前套件版本 ' + (result.currentVersion ? 'v' + result.currentVersion : '未知')
          + ' 不低于发布基线，且四个资产与 sidecar 校验一致。'
      );
    } else {
      setBadge(true);
      renderBanner(
        'info',
        '发现新版本 v' + latest.version + prerelease + previewTag,
        '通过 SHA256 交叉校验后可下载并安装；便携/未安装状态只提供发布页入口。',
        buildVersionCompare(result.currentVersion, latest.version)
      );
    }

    state.release = result;
    renderAssets(result);
    renderNotes(latest);
    renderCacheHint(result);
    renderInstallCard(result);
    // 「关于」页在拿到发布信息后显示已发布 ZIP 的 digest。
    renderPackageSha();
  }

  function renderUpdateError(error) {
    var kind = (error && error.kind) || 'httpError';
    var message = errorText(error);
    clearUpdateDetails();
    setBadge(false);
    hideInstallCard();
    renderPackageSha();

    if (kind === 'offline') {
      renderBanner('warn', '无法检查更新（离线）' + previewTag, message + '。离线时无法判断版本新旧。');
    } else if (kind === 'rateLimited') {
      var retry = error && error.retryAfterSeconds
        ? '约 ' + Math.ceil(error.retryAfterSeconds / 60) + ' 分钟后可重试。'
        : '请稍后重试。';
      renderBanner('warn', '无法检查更新（请求过于频繁）' + previewTag, message + ' ' + retry);
    } else if (kind === 'notFound') {
      renderBanner('warn', '无法检查更新' + previewTag, message);
    } else if (kind === 'invalidResponse') {
      renderBanner('err', '服务返回异常' + previewTag, message);
    } else {
      renderBanner('err', '检查更新失败' + previewTag, message);
    }

    setText('cacheHint', '上次检查失败；离线或限流时不判断版本新旧。');
  }

  function previewCheck() {
    var zipDigest = 'e90ab1897a9c0da88671c7b420b137526e5d97edee4e6139a0826f42226c7045';
    var setupDigest = 'a7b30d642c83e4e238b5b40b4977d56ea58b16352b82af116fb331d7965eeb17';
    return {
      status: 'upToDate',
      currentVersion: '0.5.11',
      latest: PREVIEW_RELEASE,
      assetsComplete: true,
      missingAssets: [],
      sidecarChecks: [
        {
          binary: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip',
          sidecar: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip.sha256',
          digestSha256: zipDigest,
          sidecarSha256: zipDigest,
          matches: true,
          error: null
        },
        {
          binary: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe',
          sidecar: 'ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe.sha256',
          digestSha256: setupDigest,
          sidecarSha256: setupDigest,
          matches: true,
          error: null
        }
      ],
      checkedAt: new Date().toISOString(),
      fromCache: false,
      nextRefreshSeconds: 600
    };
  }

  async function doCheck() {
    if (state.checking) return;
    state.checking = true;
    var button = el('btnCheck');
    if (button) {
      button.disabled = true;
      button.textContent = '检查中…';
    }
    renderBanner('info', '正在检查更新…' + previewTag, '正在获取最新版本信息，请稍候。');

    try {
      var result = hasBackend
        ? await backend.invoke('check_update', { includePrerelease: state.includePrerelease })
        : previewCheck();
      renderUpdateResult(result);
    } catch (error) {
      renderUpdateError(hasBackend ? error : { kind: 'offline', message: errorText(error) });
    } finally {
      state.checking = false;
      if (button) {
        button.disabled = false;
        button.textContent = '检查更新';
      }
    }
  }

  // ---------- install ----------

  var STAGE_LABELS = {
    precheck: '预检本地文件',
    download: '下载中',
    verify: '校验 SHA256',
    ready: '已就绪',
    handoff: '已交给更新助手',
    failed: '失败',
    cancelled: '已取消'
  };

  function artifactNameFor(version, method) {
    var target = 'x86_64-pc-windows-msvc';
    return method === 'inno'
      ? 'ruxcmd-' + version + '-' + target + '-setup.exe'
      : 'ruxcmd-' + version + '-' + target + '.zip';
  }

  function setInstallStage(stage) {
    setText('installStage', STAGE_LABELS[stage] || stage);
  }

  function setInstallBar(percent) {
    var bar = el('installBar');
    if (!bar) return;
    bar.style.width = Math.max(0, Math.min(100, percent || 0)) + '%';
  }

  function appendInstallLog(line) {
    var node = el('installLog');
    if (!node) return;
    node.hidden = false;
    node.textContent = (node.textContent ? node.textContent + '\n' : '') + line;
    var lines = node.textContent.split('\n');
    if (lines.length > 60) node.textContent = lines.slice(-60).join('\n');
    node.scrollTop = node.scrollHeight;
  }

  function renderInstallChanges(details) {
    var list = el('installChanges');
    clearNode(list);
    (details || []).forEach(function (text) {
      var item = document.createElement('li');
      item.textContent = text;
      list.appendChild(item);
    });
    list.hidden = !(details || []).length;
  }

  function installUnavailableReason(result) {
    if (!hasBackend) return '浏览器预览模式不执行下载与安装。';
    if (!result || result.status !== 'updateAvailable') return '已是最新版本，无需下载。';
    if (!result.assetsComplete) return '发布资产不完整，不能下载。';
    var checks = result.sidecarChecks || [];
    if (!checks.length || checks.some(function (check) { return check.matches !== true; })) {
      return '哈希交叉校验未完成或不一致，禁止下载。';
    }
    var method = state.install && state.install.method;
    if (method !== 'script' && method !== 'inno') {
      return '当前安装方式（' + methodLabel(method || 'unknown') + '）不支持应用内更新；请用上方按钮打开发布页手动更新。';
    }
    return null;
  }

  function renderInstallCard(result) {
    state.release = result;
    var card = el('installCard');
    if (!card) return;
    var visible = !!result && !!result.latest && result.status === 'updateAvailable';
    card.hidden = !visible;
    if (!visible) return;

    var latest = result.latest;
    setText('installVersion', 'v' + latest.version + (latest.prerelease ? '（预发布）' : ''));
    var reason = installUnavailableReason(result);
    var method = state.install && state.install.method;
    if (reason) {
      setText('installHint', reason);
      el('btnInstall').disabled = true;
      return;
    }
    if (!state.installing && !state.prepared) {
      setText(
        'installHint',
        '将下载 ' + artifactNameFor(latest.version, method)
          + ' 到 %LOCALAPPDATA%\\RuxCmd\\Desktop\\updates，校验 SHA256 后交给更新助手在 UI 退出后安装。'
      );
      el('btnInstall').disabled = false;
    }
  }

  function onUpdateStage(payload) {
    if (!payload) return;
    if (payload.id) state.activeUpdateId = payload.id;
    setInstallStage(payload.stage);
    if (payload.stage === 'download') el('installProgressWrap').hidden = false;
  }

  function onUpdateProgress(payload) {
    if (!payload) return;
    if (payload.id) state.activeUpdateId = payload.id;
    setInstallStage('download');
    el('installProgressWrap').hidden = false;
    if (payload.total) {
      setInstallBar((payload.received / payload.total) * 100);
      setText('installBytes', formatBytes(payload.received) + ' / ' + formatBytes(payload.total));
    } else {
      setText('installBytes', formatBytes(payload.received));
    }
  }

  function onUpdateFinished(payload) {
    if (!payload) return;
    if (payload.outcome === 'handedOff') {
      state.handedOff = true;
      setInstallStage('handoff');
      setText(
        'installHint',
        '更新助手已启动，将在此窗口退出后完成安装 v' + (payload.version || '')
          + (payload.rollback ? '；回滚备份：' + payload.rollback : '')
          + '。安装成功后会自动重新启动。'
      );
    }
  }

  function listenUpdateEvents() {
    if (!hasBackend || !tauriGlobal.event || typeof tauriGlobal.event.listen !== 'function') {
      return;
    }
    tauriGlobal.event.listen('update-stage', function (event) { onUpdateStage(event.payload); });
    tauriGlobal.event.listen('update-progress', function (event) { onUpdateProgress(event.payload); });
    tauriGlobal.event.listen('update-log', function (event) {
      if (event.payload && event.payload.line) appendInstallLog(event.payload.line);
    });
    tauriGlobal.event.listen('update-finished', function (event) { onUpdateFinished(event.payload); });
  }

  function renderInstallError(error) {
    var text = errorText(error);
    if (error && error.cancelled) {
      setInstallStage('cancelled');
      setText('installHint', '下载已取消，临时文件已清理。');
      appendInstallLog('已取消下载。');
      state.prepared = null;
      state.activeUpdateId = null;
      return;
    }
    setInstallStage('failed');
    setText('installHint', '更新失败：' + text);
    appendInstallLog('错误：' + text);
    toast('更新失败：' + text);
    state.prepared = null;
    state.activeUpdateId = null;
  }

  async function applyPrepared(confirmOverwrite) {
    if (!state.prepared) return;
    try {
      var outcome = await backend.invoke('apply_update', {
        id: state.prepared.id,
        confirmOverwrite: confirmOverwrite
      });
      state.handedOff = true;
      renderInstallChanges([]);
      el('btnConfirmOverwrite').hidden = true;
      el('btnInstall').disabled = true;
      setInstallStage('handoff');
      setText(
        'installHint',
        '更新助手已启动，将在本窗口退出后安装 v' + outcome.version + '；安装成功后自动重新启动。'
      );
      appendInstallLog('已交给更新助手（' + outcome.kind + '）；窗口即将关闭。');
      toast('正在关闭以完成更新…');
      var id = state.prepared.id;
      setTimeout(function () {
        backend.invoke('quit_for_update', { id: id }).catch(function (error) {
          toast('无法自动关闭：' + errorText(error) + '；请手动退出后重试。');
        });
      }, 1500);
    } catch (error) {
      if (error && error.needsConfirmation) {
        setInstallStage('ready');
        setText('installHint', '本地文件被修改或缺失，确认后才会覆盖：');
        renderInstallChanges(error.details || []);
        el('btnConfirmOverwrite').hidden = false;
        el('btnInstall').disabled = true;
        appendInstallLog('预检发现本地改动，等待显式确认。');
        return;
      }
      renderInstallError(error);
    }
  }

  async function startInstall() {
    if (state.installing) return;
    var result = state.release;
    if (!result || !result.latest) return;
    var reason = installUnavailableReason(result);
    if (reason) {
      toast(reason);
      return;
    }

    var version = result.latest.version;
    var method = state.install && state.install.method;
    if (!window.confirm(
      '将下载 ' + artifactNameFor(version, method) + '（v' + version + '），'
        + '校验 SHA256 后交给安装器。是否继续？'
    )) {
      return;
    }

    state.installing = true;
    state.prepared = null;
    state.activeUpdateId = null;
    state.handedOff = false;
    renderInstallChanges([]);
    el('installLog').hidden = true;
    el('installLog').textContent = '';
    el('installProgressWrap').hidden = false;
    setInstallBar(0);
    setText('installBytes', '');
    setInstallStage('precheck');
    el('btnInstall').disabled = true;
    el('btnCancelUpdate').hidden = false;
    el('btnConfirmOverwrite').hidden = true;
    appendInstallLog('准备更新到 v' + version + '…');

    try {
      var prepared = await backend.invoke('prepare_update', {
        version: version,
        includePrerelease: state.includePrerelease
      });
      state.prepared = prepared;
      state.activeUpdateId = prepared.id;
      setInstallBar(100);
      appendInstallLog(
        '下载完成并校验通过：' + prepared.artifact + '（' + formatBytes(prepared.size) + '）'
      );
      await applyPrepared(false);
    } catch (error) {
      renderInstallError(error);
    } finally {
      state.installing = false;
      el('btnCancelUpdate').hidden = true;
      if (!state.handedOff && !state.prepared) el('btnInstall').disabled = false;
    }
  }

  async function confirmOverwrite() {
    if (state.installing || !state.prepared) return;
    state.installing = true;
    el('btnConfirmOverwrite').disabled = true;
    try {
      await applyPrepared(true);
    } finally {
      el('btnConfirmOverwrite').disabled = false;
      state.installing = false;
      if (!state.handedOff && !state.prepared) el('btnInstall').disabled = false;
    }
  }

  async function cancelInstall() {
    var id = state.activeUpdateId;
    if (!id) {
      toast('下载尚未开始，稍后再试。');
      return;
    }
    try {
      await backend.invoke('cancel_update', { id: id });
      state.prepared = null;
      state.activeUpdateId = null;
      setInstallStage('cancelled');
      setText('installHint', '已取消本次更新，临时文件会随之清理。');
      appendInstallLog('已请求取消。');
      el('btnCancelUpdate').hidden = true;
      el('btnConfirmOverwrite').hidden = true;
      el('btnInstall').disabled = false;
    } catch (error) {
      toast('取消失败：' + errorText(error));
    }
  }

  function hideInstallCard() {
    var card = el('installCard');
    if (card) card.hidden = true;
    state.release = null;
  }

  // ---------- actions ----------

  async function openExternal(url) {
    if (!url) return;
    if (!hasBackend) {
      toast('浏览器预览模式：不会打开系统浏览器');
      return;
    }
    try {
      await backend.invoke('open_external', { url: url });
    } catch (error) {
      toast('打开链接失败：' + errorText(error));
    }
  }

  async function copyDiagnostics() {
    if (hasBackend) {
      try {
        await backend.invoke('copy_diagnostics');
        toast('诊断信息已复制到剪贴板');
      } catch (error) {
        toast('复制诊断信息失败：' + errorText(error));
      }
      return;
    }
    var text = 'RuxCmd Desktop diagnostics (browser preview, sample data)\n' +
      'manager: ' + PREVIEW_MANAGER.version + ' (' + PREVIEW_MANAGER.target + ')\n' +
      'install method: ' + PREVIEW_INSTALL.method + '\n' +
      'suite version: ' + PREVIEW_INSTALL.suiteVersion + '\n' +
      'install dir: ' + PREVIEW_INSTALL.installDir + '\n';
    try {
      await navigator.clipboard.writeText(text);
      toast('示例诊断信息已复制');
    } catch (error) {
      toast('浏览器预览模式：无法访问剪贴板');
    }
  }

  function setPrerelease(value) {
    state.includePrerelease = value;
    var checkbox = el('chkPrerelease');
    var toggle = el('switchPrerelease');
    if (checkbox) checkbox.checked = value;
    if (toggle) {
      toggle.classList.toggle('on', value);
      toggle.setAttribute('aria-pressed', value ? 'true' : 'false');
    }
  }

  var toastTimer = null;
  function toast(message) {
    var node = el('toast');
    if (!node) return;
    node.textContent = message;
    node.classList.add('show');
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(function () {
      node.classList.remove('show');
    }, 2400);
  }

  // ---------- wiring ----------

  function wire() {
    document.querySelectorAll('.nav-item').forEach(function (item) {
      item.addEventListener('click', function () {
        setView(item.getAttribute('data-view'));
      });
    });
    document.querySelectorAll('[data-url]').forEach(function (item) {
      item.addEventListener('click', function () {
        openExternal(item.getAttribute('data-url'));
      });
    });

    el('btnCheck').addEventListener('click', doCheck);
    el('btnVerify').addEventListener('click', loadVerification);
    el('btnCopy').addEventListener('click', copyDiagnostics);
    el('btnReleasePage').addEventListener('click', function () {
      openExternal(el('btnReleasePage').getAttribute('data-url'));
    });
    el('chkPrerelease').addEventListener('change', function (event) {
      setPrerelease(event.target.checked);
    });
    el('switchPrerelease').addEventListener('click', function () {
      setPrerelease(!state.includePrerelease);
    });
    el('btnInstall').addEventListener('click', startInstall);
    el('btnCancelUpdate').addEventListener('click', cancelInstall);
    el('btnConfirmOverwrite').addEventListener('click', confirmOverwrite);
    el('btnCopySha').addEventListener('click', function () {
      var hash = el('packageSha').textContent.trim();
      if (!hash || hash === '—') return;
      var copied = false;
      try {
        var area = document.createElement('textarea');
        area.value = hash;
        area.style.position = 'fixed';
        area.style.opacity = '0';
        document.body.appendChild(area);
        area.select();
        copied = document.execCommand('copy');
        document.body.removeChild(area);
      } catch (error) {
        copied = false;
      }
      toast(copied ? 'ZIP SHA256 已复制' : '复制失败，可手动选中哈希（已设为可选中）');
    });
  }

  async function init() {
    if (!hasBackend) {
      el('previewNote').hidden = false;
    }
    listenUpdateEvents();
    wire();
    await loadManagerAndInstall();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
