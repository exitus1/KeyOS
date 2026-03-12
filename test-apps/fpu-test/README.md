<!--
  SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
  SPDX-License-Identifier: GPL-3.0-or-later
-->

An FPU context switch test. Creates two concurrent threads that perform FPU-assisted calculations and OS switches between them.
The test shall fail in case of incorrect FPU register preservation and restoration during a context switch between the threads.
